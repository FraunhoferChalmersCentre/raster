import argparse
import gc
import json
import os
from sklearn import cluster, datasets, mixture, metrics
from sklearn.metrics import pairwise_distances
from sklearn.neighbors import kneighbors_graph
from sklearn.preprocessing import StandardScaler
import statistics as stats
import numpy as np
import time
import warnings

from raster_label import Raster
import data_loader


def parse_arguments():
    parser = argparse.ArgumentParser(
                description="Benchmark clustering algorithms")
    parser.add_argument('n_clusters', type=int, choices=list(range(1,6)),
                        help="Number of clusters to process (powers of 10)")
    parser.add_argument('-r', type=int, default=1, dest='repeats',
                        help="Number of repeated benchmarks")
    parser.add_argument('--metrics', '-m', action='store_true', dest='metrics',
                        help="Compute additional metrics (silhouette coefficient)")
    parser.add_argument('--pre_process', action='store_true',
                        help="Exclude pre-processing steps from benchmark timings.")

    return parser.parse_args()


def compute_cluster_metrics(alg, X):
    (name, alg) = alg
    samples = X
    if hasattr(alg, 'labels_'):
        y_pred = alg.labels_.astype(np.int)
        if name == 'RASTER' or name == 'Mean Shift':
            y_pred += 1 # allow filtering of outlier lables (-1)
            samples = X[np.nonzero(y_pred)]
            y_pred  = y_pred[np.nonzero(y_pred)]
            y_pred -= 1 # restore label values
    else:
        y_pred = alg.predict(X)

    unique_labels         = np.unique(y_pred)
    n_clusters_identified = len(unique_labels)
    #sil_score = metrics.silhouette_score(samples, y_pred, metric='euclidean', n_jobs=None)
    sil_score = None

    print(f'{len(y_pred)} total labellings')

    return (n_clusters_identified, sil_score)


def update_results(result_dict, alg_name, times):
    record = dict()
    mean  = stats.mean(times)
    stdev = stats.stdev(times) if len(times)>1 else 0.0
    record["mean"]    = mean
    record["stdev"]   = stdev
    record["timings"] = times

    print(f"mean: {mean:.6f} s")
    print(f"stdev: {stdev:.6f} s")

    result_dict[alg_name] = record


def dump_results(results, n_clusters):
    out_file = f"./output/algs_{n_clusters}clusters.json"
    with open(out_file, 'w') as f:
        json.dump(results, f, indent=1)



if __name__ == "__main__":
    args            = parse_arguments()
    nr_clusters     = int(10**args.n_clusters)
    REPEATS         = args.repeats
    pre_process     = args.pre_process
    cluster_metrics = args.metrics

    path = os.getcwd()
    os.chdir('..')
    X = data_loader.load("0_data_generators/data_{}_shuffled.csv".format(nr_clusters))
    X = np.array(X)
    print("Done loading, shape:", X.shape)
    os.chdir(path)

    if pre_process:
        # estimate bandwidth for mean shift (default parameters)
        # at least O(n²) in X's first dimension
        bandwidth = cluster.estimate_bandwidth(X, quantile=0.3, n_jobs=-1)

        # connectivity matrix for structured Ward & agglomerative
        connectivity = kneighbors_graph(
            X, n_neighbors=10, include_self=False, n_jobs=-1)
        # make connectivity symmetric
        connectivity = 0.5 * (connectivity + connectivity.T)
        print("Done preprocessing")
    else:
        print("No pre-processing")
        connectivity = None
        bandwidth = None

    # Create cluster objects
    mean_shift = cluster.MeanShift( #parallelizable with n_jobs
        bandwidth=0.1,
        bin_seeding=True,
        cluster_all=False,
        n_jobs=1)
    init_size = 500 if nr_clusters < 500 else 3*nr_clusters
    m_kmeans = cluster.MiniBatchKMeans(
        n_clusters=nr_clusters,
        init_size=init_size,
        batch_size=50)
    ward = cluster.AgglomerativeClustering(
        n_clusters=nr_clusters,
        linkage='ward',
        connectivity=connectivity)
    spectral = cluster.SpectralClustering( #parallelizable with n_jobs
        n_clusters=nr_clusters,
        eigen_solver='arpack',
        n_jobs=1,)
    dbscan = cluster.DBSCAN(eps=.3)
    affinity_propagation = cluster.AffinityPropagation(
        damping=0.9,
        preference=-0.0001)
    agglomerative = cluster.AgglomerativeClustering(
        linkage="average",
        affinity="cityblock",
        n_clusters=nr_clusters,
        connectivity=connectivity)
    birch = cluster.Birch(n_clusters=nr_clusters)
    gaus = mixture.GaussianMixture(
        n_components=nr_clusters, covariance_type='full')
    raster = Raster(precision=4, threshold=5, min_size=5)

    # uncomment algorithms if their runtime is excessive for the given dataset 
    clustering_algorithms = [
            ('RASTER', raster),
            #('DBSCAN', dbscan),
            #('MiniBatch KM', m_kmeans), # Sometimes the fastest
            ('Mean Shift', mean_shift),
            #('Birch', birch),
            #('Gauss. M.', gaus),
        ]

    # Some alogrithms do will be too slow or run out of memory with the
    # number of clusters become too large.
    if nr_clusters < 1000:
        clustering_algorithms.extend([
            ('Ward', ward),
            ('Aggl. Cl.', agglomerative),
            ])
    if nr_clusters < 100:
        clustering_algorithms.extend([
            ('Spectr. Cl.', spectral), # slow
            ('Aff. Prop.', affinity_propagation), # very slow
            ])

    results = {alg : None for (alg, _) in clustering_algorithms}

    print(f"Each algorithm runs {REPEATS} time(s)")
    for name, alg in clustering_algorithms:
        print("\nName:", name)
        times = []
        for _ in range(REPEATS):
            gc.collect()
            start = time.time()#timeit
            with warnings.catch_warnings():
                warnings.filterwarnings(
                    "ignore",
                    message="the number of connected components of the " +
                        "connectivity matrix is [0-9]+" +
                        " > 1. Completing it to avoid stopping the tree early.",
                    category=UserWarning)
                warnings.filterwarnings(
                    "ignore",
                    message="Graph is not fully connected, spectral embedding" +
                    " may not work as expected.",
                    category=UserWarning)
                alg.fit(X)
            end = time.time()#timeit
            times.append(end-start)

        update_results(results, name, times)
        # compute additional metrics on the results
        # of the last iteration/repetition
        if cluster_metrics:
            try:
                (n_clusters_found, sc) = compute_cluster_metrics((name, alg), X)
                ratio = n_clusters_found / nr_clusters
                print(f'{n_clusters_found} clusters found')
                if sc: print(f"Silhouette score: {sc:.4}")

                results[name].update(clutserratio=ratio)
                results[name].update(silhouette=sc)
            except Exception as e:
                print(f"Got error while computing silhouette coeff:\n{e}")
        dump_results(results, nr_clusters)


