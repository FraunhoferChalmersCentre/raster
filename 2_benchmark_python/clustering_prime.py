from enum import Enum

import data_loader as dl
import par_projections as pp

# retains number of observations
def map_to_tiles(points   : list,
                 precision: int ,
                 threshold: int ) -> (dict, int):
    """
    The key idea behind this function is to reduce the precision of
    spatial coordinates. These coordinates are assigned to the
    bottom-left corner of an imaginary tile, which is defined by the
    reduced precision. For instance, the tile corner (50.0212, 1.1123)
    can be used to reduce all points (50.0212__, 1.1123__) to one
    single point.

    """

    scalar    = 10 ** precision
    allPoints = dict()

    for (lat, lon) in points:

        x = int(lat * scalar)
        y = int(lon * scalar)
        point = (x, y)
        pointsInTile = allPoints.get(point, [])
        pointsInTile.append((lat, lon))
        allPoints[point] = pointsInTile

    # filter results to only retain tiles that contain at lest the
    # provided threshold value of observations
    result = dict()
    for k, v in allPoints.items():
        if len(v) >= threshold:
            result[k] = v

    return (result, scalar)


def par_map_to_tiles(points        : list, # tuples
                     precision     : int,
                     threshold     : int,
                     num_processes : int) -> (dict, int):
    """
    A parallel map_to_tiles() function
    """

    scalar     = 10 ** precision
    batches    = dl.batched_ranges(points, num_processes) # range-objects of indices
    all_points = pp.process_prime(points, batches, scalar)

    ####### all_points should now be complete #######

    # filter results to only retain tiles that contain at lest the
    # provided threshold value of observations
    result = dict()
    for k, v in all_points.items():
        if len(v) >= threshold:
            result[k] = v

    return (result, scalar)


def get_neighbors(coordinate: tuple, squares: dict) -> list:
    # neighbor lookup in O(1)

    (x, y) = coordinate
    assert isinstance(x, int)
    assert isinstance(y, int)

    # 8-way clustering
    neighbors  = [(x + 1, y    ),
                  (x - 1, y    ),
                  (x    , y + 1),
                  (x    , y - 1),
                  (x + 1, y - 1),
                  (x + 1, y + 1),
                  (x - 1, y - 1),
                  (x - 1, y + 1)]

    result = []
    for n in neighbors:
        if n in squares:
            # we know that n will be checked, so we remove it from squares
            # to prevent it from being checked again
            result.append((n, squares[n]))
            del squares[n]

    return result



def raster_clustering_tiles(squares: dict, min_size: int) -> list:
    clusters = []

    while squares:

        # pick an arbitrary point as starting point for new cluster
        k, v = squares.popitem()

        visited = dict()
        visited[k] = v

        # collect all neighbors
        to_check = get_neighbors(k, squares)

        while to_check:

            # pop a coordinate off 'to_check'; get their neighbors
            neigh_k, neigh_v  = to_check.pop()
            visited[neigh_k] = neigh_v
            to_check.extend(get_neighbors(neigh_k, squares))


        if len(visited) >= min_size:
            # add to list of clusters
            clusters.append(visited)

    return clusters
