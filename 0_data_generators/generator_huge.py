# Contraction Clustering (RASTER): Data Generator

#!/usr/bin/env python3
import os
import random


def generate_data(NUM_CLUSTERS):

    random.seed(0)

    # Clusters have 500 points that are randomly spread around a center
    points_cluster = 500

    centers        = []
    all_points     = []

    # divide plane into quadrants; pick a point in quadrant
    x_vals = [x for x in range(-1800, 1800)]
    y_vals = [x for x in range(-900, 900)]
    candidates = []

    for x in x_vals:
        for y in y_vals:
            candidates.append((x ,y))

    random.shuffle(candidates)

    vals = candidates[:NUM_CLUSTERS]

    all_points = []
    count = 0

    # Determine cluster centers
    for (x, y) in vals:

        x  = x / 10
        y  = y / 10

        x_ = x + 1
        y_ = y + 1

        margin = 0.0025
        center_x = random.uniform(x + margin, x_ - margin)
        center_y = random.uniform(y + margin, y_ - margin)

        # clusters have a fixed size
        # spread points around center (+/- z)
        size = points_cluster

        for i in range(0, size):
            # spread is random, too
            z   = random.uniform(0.0, 0.0010)
            eps = random.uniform(0.0, z)
            p1  = random.uniform(x - eps, x + eps)
            p2  = random.uniform(y - eps, y + eps)

            all_points.append((p1, p2))
            count += 1
            if (count % 100000 == 0):
              print("Points created:", count)

    f = open("data_" + str(NUM_CLUSTERS) + "_shuffled.csv", "w")
    random.shuffle(all_points)

    for (x, y) in all_points:
        f.write(str(x) + "," + str(y) + "\n")

    f.close()


if __name__ == "__main__":
  # 100,000 and 1,000,000 clusters
  generate_data(100000)
  generate_data(1000000)
