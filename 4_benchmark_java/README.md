## Running Java Benchmarks

The benchmarks rely on the [Java Microbenchmarking Harness](https://openjdk.java.net/projects/code-tools/jmh/) (JMH).
Many useful samples that demonstrate correct usage is found [here](https://hg.openjdk.java.net/code-tools/jmh/file/tip/jmh-samples/src/main/java/org/openjdk/jmh/samples/), and Javadoc for enum and annotation classes can be found [here](http://javadox.com/org.openjdk.jmh/jmh-core/1.7/allclasses-noframe.html)

This directory contains three Java/Maven projects: java-clique, java-raster and java-raster-prime. They are built separately by running

```shell

$ mvn clean package

```
in  each project's respective directory, e.g. `java-raster/`. 

Each project also has an executable script `run_benchmarks.sh` that starts a benchmark. You may give additional flags to configure the benchmark run. To see all available flags, run:

```shell

$ ./run_benchmarks.sh -h

```

If you only want a certain benchmark you should comment out those you wish to exclude in `benchmark/src/../RasterBenchmark.java` and rebuild.

To run the benchmark used for the paper, execute

```shell

$ ./run_all_benchmarks.sh

```
