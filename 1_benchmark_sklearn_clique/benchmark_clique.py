# This script is only for benchmarking CLIQUE

import argparse
import json
import os
import clique_fit
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

# test dataset
#noisy_moons = datasets.make_moons(n_samples=100, noise=.05)
#X, _ = noisy_moons

def parse_arguments():
    parser = argparse.ArgumentParser(
                description="Benchmark clustering algorithms")
    parser.add_argument('n_clusters', type=int, choices=[1,10,100,1000])
    parser.add_argument('--xsi', '-x', nargs='+', type=int,
                        help="set the xsi parameter of CLIQUE")

    parser.add_argument('-r', type=int, default=1, dest='repeats',
                        help="number of repeated benchmarks")
    parser.add_argument('--metrics', '-m', action='store_true', dest='metrics',
                        help="compute additional metrics (silhouette coefficient)")
    parser.add_argument('--no-raster', action='store_true', dest='no_raster',
                        help="run a benchmark with only CLIQUE")


    return parser.parse_args()


def compute_cluster_metrics(alg, X):
    samples = X
    if hasattr(alg, 'labels_'):
        y_pred = alg.labels_.astype(np.int)
        if name in ['RASTER', 'Mean Shift', 'CLIQUE']:
            y_pred += 1 # allow filtering of outlier lables (-1)
            samples = X[np.nonzero(y_pred)]
            y_pred  = y_pred[np.nonzero(y_pred)]
            y_pred -= 1 # restore label values
    else:
        y_pred = alg.predict(X)

    unique_labels         = np.unique(y_pred)
    n_clusters_identified = len(unique_labels)
    sil_score = metrics.silhouette_score(samples, y_pred, metric='euclidean')

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
    args = parse_arguments()
    nr_clusters     = args.n_clusters
    REPEATS         = args.repeats
    cluster_metrics = args.metrics


    path = os.getcwd()
    os.chdir('..')
    X = data_loader.load("0_data_generators/data_{}_shuffled.csv".format(nr_clusters))
    X = np.array(X)
    print("Done loading, shape:", X.shape)
    os.chdir(path)


    raster = Raster(precision=4, threshold=5, min_size=5)

    clustering_algorithms = [] if args.no_raster else [('RASTER', raster)]

    # 20 for 10 clusters, 300-500 for 100 clusters.
    # Don't even try 1000 clusters (a run takes days).
    tau = 5 / (X.size) # Clique equivalent of RASTER's threshold
    for xsi in args.xsi:
      clique = clique_fit.Clique(xsi=xsi, tau=tau)
      name   = "CLIQUE_xsi" + str(xsi)
      clustering_algorithms.append((name, clique))

    results = {alg : None for (alg, _) in clustering_algorithms}

    print(f"Each algorithm runs {REPEATS} time(s)")
    for name, alg in clustering_algorithms:
        print("\nName:", name)
        times = []
        for _ in range(REPEATS):
            start = time.time()#timeit
            alg.fit(X)
            end = time.time()#timeit
            times.append(end-start)

        update_results(results, name, times)
        # compute additional metrics on the results
        # of the last iteration/repetition
        if cluster_metrics:
            try:
                (n_clusters_found, sc) = compute_cluster_metrics(alg, X)
                print(f"{n_clusters_found} clusters found")
                print(f"Silhouette score: {sc:.4}")
                results[name].update(silhouette=sc)
            except Exception as e:
                print(f"Got error while computing silhouette coeff:\n{e}")

        dump_results(results, nr_clusters)

