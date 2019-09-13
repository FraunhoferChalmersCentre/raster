# Benchmark sequential vs parallel RASTER

import os
import statistics
import timeit

# user-defined modules
import clustering as c
import data_loader as dl
from par_projections import PARALLELIZE

REPEAT = 1
ITERATIONS = 1

def stats(bm_times):
    mean   = statistics.mean(bm_times)
    median = statistics.median(bm_times)

    return (mean, median)


if  __name__ == "__main__":
    print(f"Each benchmark runs {ITERATIONS} iterations repeated {REPEAT} times")

    #big_data = "../0_public_repo/01_input_files/data_100000_shuffled.csv"
    data     = "sample.csv"
    all_points = dl.load(data)



    print(f"|dataset| = {len(all_points)} points \n")

    ##### Step 1: Projection #####
    print("Benchmarking projection...")
    threshold = 5
    precision = 4
    num_processes = os.cpu_count()

    # Sequential
    seq_proj  = "c.map_to_tiles(all_points, precision, threshold)"
    seq_timer = timeit.repeat(seq_proj, repeat=REPEAT, number=ITERATIONS, globals=globals())
    seq_mean, seq_median = stats(seq_timer)
    print(f"(Sequential projection) avg: {seq_mean:.3f} median: {seq_median:.3f}")

    # Parallel
    for variant in PARALLELIZE:
        par_proj  = "c.par_map_to_tiles(all_points, precision, threshold, num_processes, variant)"
        par_timer = timeit.repeat(par_proj, repeat=REPEAT, number=ITERATIONS, globals=globals())
        par_mean, par_median = stats(par_timer)
        print(f"({variant} projection) avg: {par_mean:.3f} median: {par_median:.3f}")

        speedup_mean   = seq_mean / par_mean
        print(f"(Avg speedup; seq/par) {speedup_mean:.3f}")

