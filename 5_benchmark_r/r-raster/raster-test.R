### Imports and I/O #############################
rm(list=ls())
options(java.parameters = "-Xmx16000m") # increase heap size


# These imports work if you use RStudio with the provided project file
source("r-raster/raster.R")
source("load_data.R")


### Test RASTER #############################


# Test if RASTER and RASTER' has the same clusters, i.e. if they have:
# 1. the same number of clusters
# 2. corresponding clusters contain the same tiles
test_raster_vs_rasterprime <- function(c, c_prime) {
  clusters_prime[[1]][[1]]$tile == clusters[[1]][1, ]

  if (length(c) != length(c_prime)) return(FALSE)

  cluster_comp <- lapply(seq_len(length(c)), function(i) {
    if (nrow(c[[i]]) != length(c_prime[[i]])) return(FALSE)

    inner_len <- length(c_prime[[i]])
    inner_comp <- lapply(seq_len(inner_len), function(j) {
      c[[i]][j, ] == c_prime[[i]][[j]]$tile
    })

    inner_comp <- Map(function(x) all(as.logical(x)), inner_comp)
    is_equal_cluster  <- all(as.logical(inner_comp))

    return(is_equal_cluster)
  })

  cluster_comp <- Map(function(x) all(as.logical(x)), cluster_comp)
  is_all_equal <- all(as.logical(cluster_comp))

  return(is_all_equal)
}

precision <- 4
threshold <- 5
min_size  <- 4

data_dir <- file.path("..", "0_data_generators")
df       <- load_data(100, data_dir)

clusters       <- run_raster(df, precision, threshold, min_size)
clusters_prime <- run_raster_prime(df, precision, threshold, min_size)

result <- test_raster_vs_rasterprime(clusters, clusters_prime)

sprintf("Test passed? %s", result)
