# Java Raster

Built with Java 12 using Maven.

There are two projects: raster and benchmark. Both are built with:

```
mvn clean package
```

## Raster
`raster/../Raster.java` is the main class which also defines static methods that perform projection and agglomeration. Parameters are currently hard-coded in the main method.

Run with:
```shell

$ java -jar raster/target/raster.jar

```
which will print the number of found clusters.

## Raster Benchmarks

See top level README, or just run:

```shell

$ ./run_benchmakrs

```