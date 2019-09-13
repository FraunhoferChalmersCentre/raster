import math
import random
import numpy


# retains number of observations
def map_to_tiles(points   : list,
                 precision: int ,
                 threshold: int ) -> (set, int):
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

        lat = int(lat * scalar)
        lon = int(lon * scalar)
        point = (lat, lon)

        numPointsInTile  = allPoints.get(point, 0)
        allPoints[point] = numPointsInTile + 1

    # filter results to only retain tiles that contain at lest the
    # provided threshold value of observations
    result = set()
    for k, v in allPoints.items():
        if v >= threshold:
            result.add(k)

    return (result, scalar)



def get_neighbors(coordinate: tuple, squares: set) -> list:
    # neighbor lookup in O(1)
    (x, y) = coordinate

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
            # we know that n will be checked, so it is removed
            # to prevent it from being checked again
            squares.remove(n)
            result.append(n)

    return result


def clustering_tiles(squares: set, min_size: int) -> list:
    clusters = []

    while squares:

        # pick an arbitrary point as starting point for new cluster
        x = squares.pop()

        visited = set()
        visited.add(x)

        # collect all neighbors
        to_check = get_neighbors(x, squares)

        while to_check:

            # pop a coordinate off 'to_check'; get their neighbors
            val      = to_check.pop()
            visited.add(val)
            to_check.extend(get_neighbors(val, squares))


        if len(visited) >= min_size:
            # add to list of clusters
            clusters.append(visited)

    return clusters
