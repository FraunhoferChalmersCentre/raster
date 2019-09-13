"""
Contraction Clustering (RASTER):
Reference Implementation in Python with an Example
(c) 2016, 2017 Fraunhofer-Chalmers Centre for Industrial Mathematics

Algorithm development and implementation:
Gregor Ulm (gregor.ulm@fcc.chalmers.se)

Requirements:
. Python 3
. external libraries: numpy, pandas


This demo has been developed and tested on Ubuntu Linux 16.04.

For a description of the algorithm including relevant theory, please
consult our paper on Contraction Clustering (RASTER).

"""

import os
import unittest

# user-defined modules
import clustering as c
import clustering_prime as c_prime
import data_loader as dl
from   par_projections import PARALLELIZE
import raster


big_data = "../0_public_repo/01_input_files/data_1000_shuffled.csv"
data     = "input/sample.csv"

all_points = dl.load(big_data)

class TestRaster(unittest.TestCase):
    def setUp(self):
        self.threshold   = 5
        self.precision   = 4
        self.min_size    = 4

        # test [1, 2, 4, ..., cpu_count] processes
        self.num_processes = [1] + list(range(2, 1+os.cpu_count(), 2))

    def test_projection(self):
        (seq_tiles, seq_s) = c.map_to_tiles(all_points,
                                            self.precision,
                                            self.threshold)

        # all parallel projections must match the sequential one
        for variant in PARALLELIZE:
            for cores in self.num_processes:
                (par_tiles, par_p) = c.par_map_to_tiles(all_points,
                                                        self.precision,
                                                        self.threshold,
                                                        cores,
                                                        variant)
                self.assertEqual(seq_s, par_p,
                                "sequential and parallel used different scaling factors")
                self.assertSetEqual(seq_tiles, par_tiles,
                                    "sequential and parallel versions returned different tile projections")


        # raster_prime must project to the same tiles as raster
        (prime_tiles, s_prime) = c_prime.map_to_tiles(all_points,
                                                self.precision,
                                                self.threshold)

        self.assertEqual(seq_s, s_prime,
                        "sequential and parallel used different scaling factors")
        prime_set = set(prime_tiles.keys())
        self.assertSetEqual(seq_tiles, prime_set,
                        "raster and rester_prime returned different tile projections")

        for cores in self.num_processes:
            # parallel raster_prime must project to the same tiles as raster
            (prime_tiles, s_prime) = c_prime.par_map_to_tiles(all_points,
                                                        self.precision,
                                                        self.threshold,
                                                        cores)

            self.assertEqual(seq_s, s_prime,
                            "sequential and parallel used different scaling factors")
            prime_set = set(prime_tiles.keys())
            self.assertSetEqual(seq_tiles, prime_set,
                            "raster and parallel rester_prime returned different tile projections")


    def test_clusters(self):
        (seq_raster, _) = raster.raster(all_points,
                                       self.precision,
                                       self.threshold,
                                       self.min_size)

        # raster and raster_prime must give the same clusterings
        (clusters_prime, _) = raster.raster_prime(
                                               all_points,
                                               self.precision,
                                               self.threshold,
                                               self.min_size)

        self.cluster_prime_test(seq_raster, clusters_prime)

        ## Visually inspect that we have stored what we expect
        #print(clusters_prime[0].popitem())

        # Parallel implementation should be invariant to the number of
        # cores used
        for cores in self.num_processes:
            # parallel raster must give the same clusters as sequential raster
            (par_raster, _) = raster.par_raster(all_points,
                                               self.precision,
                                               self.threshold,
                                               self.min_size ,
                                               cores)

            self.assertCountEqual(seq_raster, par_raster,
                                  "different clusterings found")

            # raster and par_raster_prime must give the same clusterings
            (clusters_prime, _) = raster.par_raster_prime(
                                                   all_points,
                                                   self.precision,
                                                   self.threshold,
                                                   self.min_size,
                                                   cores)

            self.cluster_prime_test(seq_raster, clusters_prime)



    def cluster_prime_test(self, clusters, clusters_prime):
        prime_keys = [k.keys() for k in clusters_prime]
        self.assertCountEqual(clusters, prime_keys,
                        "different clusterings found")


if __name__ == "__main__":
    unittest.main()

