# Benchmark sequential vs parallel RASTER

import gc
import copy
import csv
import statistics
import os
from time import time

# user-defined modules
import data_loader as dl
import clustering as c
import clustering_prime as c_prime


REPEATS    = 5
ITERATIONS = 1

CSV_HEADER = ['#clusters', '#clusters identified [ratio]', '#cores',
              'mean [s]',
              'sample std_dev [s]',
              'mean projection [s]',
              'mean clustering [s]',
              'sample std_dev projection [s]',
              'sample std_dev clustering [s]',
              'times projection [s]',
              'times clustering [s]']


def stats(bm_times):
    mean  = statistics.mean(bm_times)
    stdev = statistics.stdev(bm_times)

    return (mean, stdev)


def output_result(name, threshold, precision, min_size, result):
    output_file = name + "_benchmark_precision{}_threshold{}_minsize{}.csv"
    output_file = output_file.format(precision, threshold, min_size)

    with open(f"./output/{output_file}", 'a', newline='') as csvfile:
        writer = csv.writer(csvfile, delimiter=';')
        writer.writerow(result)


def format_and_output(projection_results,
                      clustering_results,
                      output_args,
                      n_clusters,
                      cores):

    clusters_found, cluster_timings = clustering_results
    projection_timings              = projection_results

    assert(len(cluster_timings)    == REPEATS)
    assert(len(projection_timings) == REPEATS)

    # found the same number of clusters in all repeats
    assert(all([x == clusters_found[0] for x in clusters_found]))
    clusters_found = clusters_found[0]

    total_times = list(map(lambda x: x[0]+x[1], zip(cluster_timings, projection_timings)))

    percent            = clusters_found / n_clusters
    avg_c, stdev_c     = stats(cluster_timings)
    avg_p, stdev_p     = stats(projection_timings)
    avg_tot, stdev_tot = stats(total_times)

    record = [n_clusters, percent, cores,
              avg_tot, stdev_tot,
              avg_p, avg_c, stdev_p, stdev_c,
              list(projection_timings), list(cluster_timings)]
    output_result(*output_args, record)


def bm(bm_fun, bm_args):
    results = []
    timings = []
    for _ in range(REPEATS):
        result, timing = timer(bm_fun, bm_args)
        print(f"{bm_fun.__name__} returned {len(result)} values; {timing}s")
        results.append(result)
        timings.append(timing)
    return (results, timings)


def bm_split(fun_proj, args_proj, fun_clust, min_size):
    results_proj  = []
    results_clust = ([], [])
    for _ in range(REPEATS):
        gc.collect()

        proj_res, proj_timing = timer(fun_proj, args_proj) #timed call
        results_proj.append(proj_timing)

        tiles = proj_res[0]
        n_tiles = len(tiles)
        clust_res, clust_timing = timer(fun_clust, (tiles, min_size)) #timed call

        results_clust[0].append(len(clust_res))
        results_clust[1].append(clust_timing)
        print(f"Projection/Clustering: {n_tiles}/{len(clust_res)}; {proj_timing:.3f}s/{clust_timing:.3f}s")
        del proj_res
        del clust_res

    return (results_proj, results_clust)


def timer(fun, args):
    t0 = time()
    result = fun(*args)
    t1 = time()

    fun_time = t1-t0
    return (result, fun_time)


def sequential_bm():
    threshold = 5
    min_size  = 4

    precisions = [3]
    n_clusters = [int(1e1), int(1e2)]
    # n_clusters = [int(1e2), int(1e3), int(1e4), int(1e5)]
    filename = "raster_python"

    # Write Header to result file
    for precision in precisions:
        output_args = (filename, threshold, precision, min_size)
        output_result(*output_args, CSV_HEADER)

    data_files  = [f"data_{n}_shuffled.csv" for n in n_clusters]
    for i, file in enumerate(data_files):

        path = os.getcwd()
        os.chdir('..')
        loc =  f"0_data_generators/{file}"
        all_points = dl.load(loc)
        print(f"|dataset| = {len(all_points):,} points \n")
        os.chdir(path)

        for precision in precisions:
            print(f"(raster) precision {precision} ...")

            proj_args           = (all_points, precision, threshold)
            proj_res, clust_res = bm_split(c.map_to_tiles, proj_args,
                                           c.raster_clustering_tiles, min_size)
            print("")

            output_args = (filename, threshold, precision, min_size)
            format_and_output(proj_res,
                              clust_res,
                              output_args,
                              n_clusters[i],
                              cores=1)

def parallel_bm():
    threshold = 5
    min_size  = 4

    precisions = [3, 4]
    #n_clusters = [int(1e4), int(1e5)]
    n_clusters = [int(1e1), int(1e2)]
    
    n_processes = [1,2,4,6]
    filename = "par_raster_prime_python"

    # Write Header to result file
    for precision in precisions:
        output_args = (filename, threshold, precision, min_size)
        output_result(*output_args, CSV_HEADER)

    data_files  = [f"data_{n}_shuffled.csv" for n in n_clusters]
    for i, file in enumerate(data_files):

        path = os.getcwd()
        os.chdir('..')
        loc =  f"0_data_generators/{file}"
        all_points = dl.load(loc)
        print(f"|dataset| = {len(all_points):,} points \n")
        os.chdir(path)
        
        for precision in precisions:
            for cores in n_processes:
                print(f"(par_raster_prime) {cores} cores and precision {precision} ...")
                proj_args = (all_points, precision, threshold, cores)
                proj_res, clust_res = bm_split(c_prime.par_map_to_tiles, proj_args,
                                               c_prime.raster_clustering_tiles, min_size)
                print("")

                ## output result
                output_args = (filename, threshold, precision, min_size)
                format_and_output(proj_res,
                                  clust_res,
                                  output_args,
                                  n_clusters[i],
                                  cores=cores)


def bm_all():
    print(f"Each benchmark runs {ITERATIONS} iterations repeated {REPEATS} times\n")
    sequential_bm()
    parallel_bm()


if  __name__ == "__main__":
  bm_all()
