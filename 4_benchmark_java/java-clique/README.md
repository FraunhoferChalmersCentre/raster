# Java Clique

Built with Java 12 using Maven.

There are two sub-projects: clique and benchmark. Both are built with:

```
mvn clean package
```

## Prerequisite

We depend on a third-party Weka implementation of Clique that has been extracted from the [subspace R package](https://cran.r-project.org/package=subspace) (version 1.0.4). We require you to download the source archive and extract `i9-subspace.jar` and `i9-weka.jar` found in `/subspace/inst/java/` of the archive.

To successfully build the projects, you also have to install `i9-subspace.jar` and `i9-weka.jar` in your local Maven repository. To correctly install them, please run the provided installation script after having downloaded and replaced the placeholders with the actual `.jar` files:

```shell

$ ./install_subspace.sh

```

## Clique
`clique/../Clique.java` is the main class which also defines static methods for loading data and Clique clustering. The input file and algorithm parameters are hardcoded to 

Run the main class with:
```shell

$ java -jar clique/target/clique.jar

```

which will print the number of clusters found in the 2-D plane.

### Cluster data

Since we use a Weka algorithm, it is convenient if input data is stored in `.arff` format. Converting our input files from csv to arff is just a matter of adding a few annotations to the top of the file. We have converted the input file with 10 clusters to `.arff` format in `clique/resources/`. Simply add the six first lines, i.e. all starting with '@', of this file to any of the other csv input files and save it to `clique/resources/` with a `.arff` extension.

Weka is also able to read csv, but assumes that the first line is a header. Since our files do not have a header, we have to modify our files in any case.

## Clique Benchmarks

See top level README, or just run:

```shell

$ ./run_benchmarks.sh

```