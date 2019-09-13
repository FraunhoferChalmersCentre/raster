import os
import numpy as np

# user-defined modules
import clustering as c


class Raster:

  def __init__(self, threshold, min_size, precision):
      self.threshold = threshold
      self.min_size  = min_size
      self.precision = precision

  def fit(self, X, y=None, sample_weight=None):

      ## Step 1: Projection
      threshold            = self.threshold
      precision            = self.precision
      (tiles_dict, scalar) = c.mapToTiles_Tiles(X, precision, threshold)
      tiles                = tiles_dict.keys()

      ## Step 2: Agglomeration
      min_size = self.min_size
      clusters = c.raster_clustering_tiles(tiles, min_size)
      print("Number of clusters: ", len(clusters))


      # key: unscaled input coordinate, value: cluster label
      full = dict()

      labels = []
      values = []

      output = []
      count  = 0 # change for scikit-learn

      # assign a numerical label to each cluster
      for cluster in clusters:

          for (x, y) in cluster:
              # look up tile in tiles_dict
              full[(x, y)] = count

          count += 1

      # Initially, all samples are noise.
      labels = np.full(X.shape[0], -1, dtype=np.intp)

      # assign label to values; lookup X[i] in results_dict

      all_keys = full.keys()
      for i in range(len(X)):
        val       = X[i]

        # scale and look-up
        a = int(val[0] * scalar)
        b = int(val[1] * scalar)

        if (a, b) not in all_keys:
          continue # leave existing valure at -1
        else:
          labels[i] = full[(a, b)]

      self.labels_ = labels
