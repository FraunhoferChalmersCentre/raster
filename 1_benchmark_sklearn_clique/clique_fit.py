import clique.Clique as clique
import clique.CliqueOpt as clique_opt
import numpy as np

class Clique:
    """
    Clique partitions the feature space into xsi equal parts in each
    dimension, where the intersection of one interval from each
    dimension is called unit.

    If a unit contains more than tau parts of all the data points, then
    it is a dense unit.

    Clusters are the maximal sets of connected dense units
    """
    def __init__(self, xsi, tau, optimized=True):
       global clique
       if optimized: clique = clique_opt

       self.xsi = xsi
       self.tau = tau

    def fit(self, X):
        clusters = clique.run_clique(X, self.xsi, self.tau)
        clique.save_to_file(clusters, "../output/clique_output.txt")

        two_dim_clusters = filter(lambda c: len(c.dimensions) == 2, clusters)

        labels = np.full(X.shape[0], -1, dtype=np.intp)

        # assign label to values; lookup X[i] in results_dict
        for labl, c in enumerate(two_dim_clusters):
            for index in c.data_point_ids:
                # Dict miss means that the point is an outlier.
                # outliers are assigned a -1 label
                labels[index] = labl

        self.labels_ = labels

        return clusters

