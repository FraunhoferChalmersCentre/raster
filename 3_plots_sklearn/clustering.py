import math
import random
import numpy


# retains number of observations
def mapToTiles_Tiles(points   : list,
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

    print(points)


    #for (lat, lon) in points:
    for x in points:

        a = x[0]
        b = x[1]
        lat = int(a * scalar)
        lon = int(b * scalar)

        if (lat, lon) in allPoints.keys():
            allPoints[(lat, lon)] += 1

            #allPoints[(lat, lon)].append((a, b))

        else:
            allPoints[(lat, lon)] = 1

            #allPoints[(lat, lon)]  = [(a, b)]


    # filter results to only retain tiles that contain at least the
    # provided threshold value of observations

    result = dict()
    for k in allPoints.keys():
        vals = allPoints[k]
        #if len(vals) >= threshold:
        if vals >= threshold:
            result[k] = vals

    return (result, scalar)



def getNeighbors(coordinate: tuple, squares: set):
    # neighbor lookup in O(1)
    result = []
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

    for n in neighbors:
        if n in squares:
            squares.remove(n) # side effect!
            result.append(n)

    return result


def raster_clustering_tiles(squares: list, min_size: int):

    squares = set(squares)
    clusters = []

    while not (len(squares) == 0):
        # pick an arbitrary point as starting point for new cluster
        x        = squares.pop()
        visited  = set()
        visited.add(x)

        # collect all neighbors
        to_check = getNeighbors(x, squares)

        while not (to_check == []):
            # pop a coordinate off 'to_check'; get their neighbors
            val = to_check.pop()
            visited.add(val)
            to_check.extend(getNeighbors(val, squares))

        # remove all points from set
        # This is now done in getNeighbors
        if len(list(visited)) >= min_size:
            # add to list of clusters
            clusters.append(visited)

    return clusters
