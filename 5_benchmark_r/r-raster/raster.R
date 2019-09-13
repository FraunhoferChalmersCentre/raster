### Imports and I/O #############################
rm(list=ls())
options(java.parameters = "-Xmx16000m") # increase heap size

library(dplyr) # we use dplyr::count and grouped compuptations

# This import works if you use RStudio with the provided project file
source("load_data.R")


to_key <- function(point) {
   return(to_key2(point[[1]], point[[2]]))
}

to_key2 <- function(x, y) {
  return(sprintf("%d;%d", x, y))
}

to_point <- function(key) {
  tmp <- strsplit(key, ";")
  x   <- as.integer(tmp[[1]][1])
  y   <- as.integer(tmp[[1]][2])

  return(c(x, y))
}


### Projection #############################

map_to_tiles <- function(df, precision, threshold) {
  projections <- trunc(df * (10^precision))
  projections <- count(projections,  V1, V2)

  projections <- subset(projections, n >= threshold)

  return(projections[, 1:2])
}

df_to_hashmap <-function(df) {
  hashmap <- new.env(hash=TRUE, parent=emptyenv())
  apply(df, 1, function(row) hashmap[[to_key(row)]] <- t(row))

  return(hashmap)
}


### Projection Prime #############################

map_to_tiles_prime <- function(df, precision, threshold) {
  projections <- trunc(df * (10^precision))
  concat <- data.frame(df, projections)
  hashmap <- new.env(hash=TRUE, parent=emptyenv())

  res <- concat %>%
    group_by(V1.1, V2.1) %>%
    filter(n() >= threshold) %>%
    group_map(~ merge_group(.x, hashmap), keep=TRUE)

  return(hashmap)
}

merge_group <- function(group, hashmap) {
  key <- to_key2(group$V1.1[1], group$V2.1[2])
  hashmap[[key]] <- list(group$V1, group$V2)
}


### Clustering #############################

get_neighbors <- function(tiles, coordinate) {
  x = coordinate[1]
  y = coordinate[2]

  xs = c(x + 1,
         x - 1,
         x    ,
         x    ,
         x + 1,
         x + 1,
         x - 1,
         x - 1)

  ys = c(y   ,
         y   ,
         y + 1,
         y - 1,
         y - 1,
         y + 1,
         y - 1,
         y + 1)

  # lookup neighbors
  neighbor_keys  <- mapply(to_key2, xs, ys)
  neighbors <- lapply(neighbor_keys, function(x) tiles[[x]])

  # remove lookup misses
  non_null  <- which(sapply(neighbors, function(x) !is.null(x)))
  neighbors <- neighbors[non_null]

  # remove neighboring tiles since we know that they will be visited
  rm(list=neighbor_keys[non_null], envir=tiles, inherits=FALSE)

  return(neighbors)
}



raster_clustering_tiles <- function(tiles, min_size) {
  clusters <- list()
  all_keys <- ls(tiles)

  for (key in all_keys) {
    tile    <- tiles[[key]]
    if (is.null(tile)) next # this tile has already been visited
    rm(list=key, envir=tiles, inherits=FALSE)

    cluster  <- tile
    to_check <- get_neighbors(tiles, tile)

    while (length(to_check) > 0) {
      neighbor_tile <- to_check[[1]]
      to_check      <- to_check[-1]

      cluster       <- rbind(cluster, neighbor_tile)

      snd_neighbors <- get_neighbors(tiles, neighbor_tile)
      to_check      <- append(to_check, snd_neighbors)
    }

    if (nrow(cluster) >= min_size) {
      clusters <- append(clusters, list(cluster))
    }
  }
  return(clusters)
}


### Clustering Prime #############################

get_neighbors_prime <- function(tiles, coordinate) {
  x = coordinate[[1]]
  y = coordinate[[2]]


  neighbors = list(c(x+1L, y)  ,
                   c(x-1L, y)  ,
                   c(x  , y+1L),
                   c(x  , y-1L),
                   c(x+1L, y-1L),
                   c(x+1L, y+1L),
                   c(x-1L, y-1L),
                   c(x-1L, y+1L))

  # lookup neighbors
  neighbor_keys  <- sapply(neighbors, to_key)
  neighbor_vals  <- lapply(neighbor_keys, function(x) tiles[[x]])

  # remove lookup misses
  non_null  <- which(sapply(neighbor_vals, function(x) !is.null(x)))
  neighbors <- neighbors[non_null]
  neighbor_vals <- neighbor_vals[non_null]

  # remove neighboring tiles from global hashmap since we know that they will be visited
  rm(list=neighbor_keys[non_null], envir=tiles, inherits=FALSE)

  # Prepare a list of exsisting neighbor tiles
  neighbors <- lapply(seq_len(length(neighbors)), function(i) {
    list(tile=neighbors[[i]], points=neighbor_vals[[i]])
  })

  return(neighbors)
}

raster_clustering_tiles_prime <- function(tiles, min_size) {
  clusters <- list()
  all_keys <- ls(tiles)

  for (key in all_keys) {
    tile_points <- tiles[[key]]
    if (is.null(tile_points)) next # this tile has already been visited
    rm(list=key, envir=tiles, inherits=FALSE)

    cluster  <- list(list(tile=to_point(key), points=tile_points))
    to_check <- get_neighbors_prime(tiles, cluster[[1]]$tile)

    while (length(to_check) > 0) {
      neighbor_tile <- to_check[[1]]
      to_check      <- to_check[-1]

      cluster       <- append(cluster, list(neighbor_tile))

      snd_neighbors <- get_neighbors_prime(tiles, neighbor_tile$tile)
      to_check      <- append(to_check, snd_neighbors)
    }

    if (length(cluster) >= min_size) {
      clusters <- append(clusters, list(cluster))
    }
  }
  return(clusters)
}


### Run RASTER #############################

run_raster <- function(df, precision, threshold, min_size) {
  tiles    <- map_to_tiles(df, precision, threshold)
  tiles    <- df_to_hashmap(tiles)
  clusters <- raster_clustering_tiles(tiles, min_size)

  return(clusters)
}

run_raster_prime <- function(df, precision, threshold, min_size) {
  tiles    <- map_to_tiles_prime(df, precision, threshold)
  clusters <- raster_clustering_tiles_prime(tiles, min_size)

  return(clusters)
}

precision <- 4
threshold <- 5
min_size  <- 4

data_dir <- file.path("..", "0_data_generators")
df       <- load_data(100, data_dir)

clusters       <- run_raster(df, precision, threshold, min_size)
clusters_prime <- run_raster_prime(df, precision, threshold, min_size)
