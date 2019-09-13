"""
Contraction Clustering (RASTER):
Reference Implementation in Python with an Example

Requirements:
. Python 3
. external libraries: numpy, pandas
"""

import os

import numpy as np

# user-defined modules
import clustering as c


class Raster:

  def __init__(self, precision=1, threshold=1, min_size=1):
      self.precision = precision
      self.threshold = threshold
      self.min_size  = min_size

  def fit(self, X, y=None, sample_weight=None):

      ## Step 1: Projection
      threshold = self.threshold
      precision = self.precision
      (tiles, scalar) = c.map_to_tiles(X, precision, threshold)

      ## Step 2: Agglomeration
      min_size = self.min_size

      clusters = c.clustering_tiles(tiles, min_size)

      # change for scikit-learn:
      # key: scaled input coordinate (i.e. tile), value: cluster label
      tile_to_label = dict()

      # assign a numerical label to each cluster
      for count, cluster in enumerate(clusters):
          for (x, y) in cluster:
              tile_to_label[(x, y)] = count

      labels = np.empty(X.shape[0], dtype=np.intp)

      # assign label to values; lookup X[i] in results_dict
      for i, val in enumerate(X):
        # scale and look-up
        a = int(val[0] * scalar)
        b = int(val[1] * scalar)

        # Dict miss means that the point is an outlier
        # are assigned a -1 label
        labl = tile_to_label.get((a,b), -1)
        labels[i] = labl

      self.labels_ = labels
