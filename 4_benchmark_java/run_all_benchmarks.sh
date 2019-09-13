#! /bin/bash

BM_DIRS=("java-raster/" "java-raster-prime/" "java-clique")

OUTPUT_FILE="./bm_output.txt"
BM_COMMAND="run_benchmarks.sh -f 1 -o $OUTPUT_FILE"

BASE_DIR=$(pwd)

for f in ${BM_DIRS[@]}; do
  cd $f

  echo "Building..."
  mvn clean package

  echo "Benchmarking in $f..."
  sh $BM_COMMAND

  cd $BASE_DIR
done
