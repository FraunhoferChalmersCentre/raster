import os

# user-defined modules
import data_loader as dl
import clustering as c
import clustering_prime as c_prime


def raster(all_points, precision, threshold, min_size):
    ## Step 1: Projection
    (tiles, scalar) = c.map_to_tiles(all_points, precision, threshold)

    ## Step 2: Agglomeration
    clusters = c.raster_clustering_tiles(tiles, min_size)

    return (clusters, scalar)


def par_raster(all_points, precision, threshold, min_size, num_processes):
    ## Step 1: Projection (Parallel)
    (tiles, scalar) = c.par_map_to_tiles(all_points,
                                         precision ,
                                         threshold ,
                                         num_processes)

    ## Step 2: Agglomeration (Sequential)
    clusters = c.raster_clustering_tiles(tiles, min_size)

    return (clusters, scalar)


def raster_prime(all_points, precision, threshold, min_size):
    ## Step 1: Projection
    (tiles, scalar) = c_prime.map_to_tiles(all_points, precision, threshold)

    ## Step 2: Agglomeration
    clusters = c_prime.raster_clustering_tiles(tiles, min_size)

    return (clusters, scalar)


def par_raster_prime(all_points, precision, threshold, min_size, num_processes):
    ## Step 1: Projection (Parallel)
    (tiles, scalar) = c_prime.par_map_to_tiles(all_points,
                                         precision ,
                                         threshold ,
                                         num_processes)

    ## Step 2: Agglomeration (Sequential)
    clusters = c_prime.raster_clustering_tiles(tiles, min_size)

    return (clusters, scalar)



if __name__ == "__main__":

    # load input data
    data_path = "../0_data_generators/output/data_1000_shuffled.csv"
    all_points = dl.load(data_path)

    """
    1) RASTER clusters

    RASTER projects points to tiles and disregards the former after the
    projection has been performed. Thus, it requires merely constant
    space, assuming bounded integers or a bounded coordinate system like
    the GPS coordinate system for our planet.

    Input is projected to points that represent tiles.

    """
    precision = 3
    threshold = 5
    min_size  = 4

    clusters, scalar = raster(all_points, precision, threshold, min_size)
    print("Number of clusters: ", len(clusters))


    output = []
    count  = 1
    for cluster in clusters:

        for (x, y) in cluster:
            x = x / scalar
            y = y / scalar
            output.append((count, x, y))

        count += 1

    output_file = "clustered.csv"
    print("Writing output to: ", output_file)

    with open(output_file, "w") as f:
        f.write("Cluster Number, X-Position, Y-Position\n")
        for (num, x, y) in output:
            f.write(str(num) + ", " + str(x) + ", " + str(y) + "\n")


