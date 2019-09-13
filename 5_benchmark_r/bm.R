### Imports and I/O #############################
rm(list=ls())
options(java.parameters = "-Xmx16000m") # increase heap size

library(microbenchmark)
library(subspace)

# These imports work if you use RStudio with the provided project file
source("r-raster/raster.R")
source("load_data.R")


### Utilites #############################

is_highest_dim_cluster <- function(cluster) {
  return(all(cluster$subspace))
}

# Abuse the check function for side-effects
print_clusters_found <- function(vals){
  raster  <- vals[[1]]
  rasterp <- vals[[2]]
  clique  <- Filter(is_highest_dim_cluster, vals[[3]])

  r_str  <- sprintf("RASTER found %d clusters", length(raster))
  rp_str <- sprintf("RASTER' found %d clusters", length(rasterp))
  c_str  <- sprintf("CLIQUE found %d clusters", length(clique))

  cat(r_str, rp_str, c_str, sep="\n")

  return(TRUE)
}


### Benchmarks #############################

num_clusters <- c(10L, 100L, 1000L)
data_dir <- file.path("..", "0_data_generators")

precisions <- c(3.5, 4)
threshold <- 5L
min_size  <- 4L

xi <- c(20L, 300L, 1000L)
tau <- 5 / nrow(df)

for (precision in precisions) {
  res <<- lapply(seq_len(length(num_clusters)), function(i){
    df  <- load_data(num_clusters[i], dir = data_dir)

    cat(sprintf("Using %d-clusters and RASTER with p=%.1f and CLIQUE with xi=%d\n", num_clusters[i], precision, xi[i]))
    r <- microbenchmark(run_raster(df, precision, threshold, min_size),
                        run_raster_prime(df, precision, threshold, min_size),
                        CLIQUE(df, xi[i], tau),
                        check=print_clusters_found,
                        times=5L)
    print(r)
    cat("\n\n")
  })
}

