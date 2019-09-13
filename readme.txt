Sequential and Parallel Contraction Clustering:
    Reference Implementations and Benchmarks
(c) 2016 - 2019 Fraunhofer-Chalmers Research Centre
                for Industrial Mathematics (FCC), Gothenburg, Sweden

Research and development by Gregor Ulm, Simon Smith, Adrian Nilsson,
Emil Gustavsson, and Mats Jirstrand.

This repository contains artifacts accompanying our journal paper on
RASTER. Two of these artifacts were not used for the final revision of
the paper, i.e. a parallel implementation of RASTER in Python and a
sequential implementation of RASTER in R.

The content is as follows:

/0_data_generators
 Data generators for creating input files with GPS coordinates.

/1_benchmark_sklearn
 Benchmark that compares RASTER with several other clustering
 algorithms. The entry point is benchmark.py.

/1_benchmark_sklearn_clique
 Separate benchmark for a Python implementation of CLIQUE. The entry
 point is benchmark_clique.py. The source code file CliqueOPT.py is
 our modification of CLIQUE.py, which led to a signficant performance
 increase.

/2_benchmark_python
 Benchmark for sequential and parallel implementations of RASTER in
 Python. The designated entry point is bm.py. The file bm_projection.py
 can be used for separately benchmarking the projection step. Note that
 the parallel implementation of RASTER in Python has not been optimized
 and is not referenced in our paper.

/2_benchmark_rust
 Benchmark for sequential and parallel implementations of RASTER in
 Python. The entry point is /src/main.rs.

/3_comparison_sklearn
 Qualitative comparison of several clustering algorithms. The entry
 point is plot_comparison.py.

/A-1_plots_sklearn
 Additional plots for qualitative assessment of RASTER. The entry
 points are plot_raster_parameters.py and plot_cluster_dataset_3.py.

/A-2_benchmark_java
 Benchmark of Java implementations of RASTER and CLIQUE. The entry
 point is run_all_benchmarks.sh, which successively runs benchmarks of
 CLIQUE, RASTER and RASTER'.

/Z_benchmark_r
 Benchmark of an R implementation of RASTER. The entry point is bm.R.
 This implementation is complete, but it is not referenced in our
 paper.

The prefix in the file name refers to the accompanying paper. The
directories with a leading 1, 2, or 3 refer to the main experiments
1, 2, and 3 as described in Chapter 4. The directory prefixed with A-1
and A-2 relate to Appendix 1 and Appendix 2. The data generator was
used for all three main experiments. Lastly, we added an implementation
of RASTER in R. This is an artifact we produced during our research.

The scripts in /3_plots_sklearn are a good starting point for
experimenting with RASTER.

All our code is released under an MIT license. However, some of the
scripts in this repository are modifications of existing code from
scikit-learn [1], which uses the BSD license. The Python implementation
of CLIQUE is a modification based on a publicly available implementation
by György Katona [2], which has been released under the MIT license.
Lastly, one of the scripts imports JAR files from the R library
subspace [3]. Among others, those files contain a Java implementation of
CLIQUE. That code has originally been released under the GPL-2 license
and is therefore NOT included in this code respository. The missing
files are available on Github [4, 5]. Placeholder files in this
repository indicate where those JAR files have to be put.

[1] https://github.com/scikit-learn/scikit-learn
[2] https://github.com/georgekatona/Clique
[3] https://github.com/matthhan/subspace
[4] https://github.com/matthhan/subspace/blob/master/inst/java/i9-weka.jar
[5] https://github.com/matthhan/subspace/blob/master/inst/java/i9-subspace.jar
