
load_data <- function(n_clusters, dir) {
  filename <- paste("data_", n_clusters, "_shuffled.csv", sep="")
  path <- file.path(data_dir, filename)
  
  df <- read.csv(path, header = FALSE)
  return(df)
}