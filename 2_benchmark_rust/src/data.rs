use raster::{Float, Point, TileSet};
use rayon::prelude::*;
use std::error::Error;
use std::fs;
use std::path::Path;



#[allow(dead_code)]
/// Reads a CSV file without header and two real numbers per row (e.g. "10.42, 1080.360").
pub fn read<P: AsRef<Path>>(path: P) -> Result<Vec<Point>, Box<Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_path(path)?;
    let iter = rdr.deserialize();

    let mut data: Vec<Point> = Vec::new();
    for row in iter {
        let point = row?;
        data.push(point);
    }
    Ok(data)
}


/// Reads a CSV file without header and two real numbers per row (e.g. "10.42, 1080.360").
pub fn parallel_read<P: AsRef<Path>>(path: P) -> Result<Vec<Point>, Box<Error>> {
    let contents = fs::read_to_string(path)?;

    Ok(
        contents.par_lines()
            .map(|line| {
                let mut split = line.split(",").map(|s| s.to_string());
                let x: String = split.next().expect("No first argument.");
                let y: String = split.next().expect("No second argument.");
                // Parsing f64 is faster than f32
                let x: f64 = x.parse().expect("Couldn't parse first argument.");
                let y: f64 = y.parse().expect("Couldn't parse second argument.");
                Point::new(x as Float, y as Float)
            })
            .collect()
    )
}


#[allow(dead_code)]
/// Write clusters to output/clustered.csv containing a cluster label for each tile.
pub fn write_clusters(clusters: Vec<TileSet>, scalar: Float) -> Result<(), Box<Error>> {
    let mut wtr = csv::Writer::from_path("output/clustered.csv")?;

    let mut cluster_nr = 1;
    for c in clusters {
        for (x, y) in c {
            let x = (x as Float / scalar).to_string();
            let y = (y as Float / scalar).to_string();
            wtr.write_record(&[cluster_nr.to_string(), x, y])?;
        }
        cluster_nr += 1;
    }

    wtr.flush()?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexSet;

    #[test]
    fn par_eq_test() {
        let points = read("../0_data_generators/data_100_shuffled.csv").unwrap();

        let (tiles_seq, _) = raster::map_to_tiles(&points, 4., 5);

        let tiles_par: raster::TileSet = raster::par::map_to_tiles(&points, 4., 5, 4).0.collect();

        assert_eq!(tiles_seq, tiles_par);
    }

    #[test]
    fn test_two_core_map_to_tiles_slices(){
        let points = parallel_read("../0_data_generators/data_100_shuffled.csv").unwrap();
        let precision = 5.;
        let threshold = 5;
        let (left, right, _) = raster::par::dual_clustering::map_to_tile_slices(&points, precision, threshold, num_cpus::get());
        // regular mapping should give the correct result
        let (tiles, _) = raster::map_to_tiles(&points, precision, threshold);

        let intersect = &left & &right;
        assert_eq!(intersect, IndexSet::new());

        assert_eq!(tiles.len(), left.len()+right.len());
        let union = &left | &right;
        assert_eq!(tiles, union);
    }

    #[test]
    fn test_n_core_map_to_tiles_slices(){
        let points = parallel_read("../0_data_generators/data_100_shuffled.csv").unwrap();
        let precision = 5.;
        let threshold = 1;
        let (tiles, scalar) = raster::par::map_to_tiles(&points, precision, threshold, num_cpus::get());
        let tile_slices = raster::par::split_vertically(tiles, -180, 180, scalar, num_cpus::get());
        // regular mapping should give the correct result
        let (tiles, _) = raster::map_to_tiles(&points, precision, threshold);

        assert_eq!(tiles.len(), tile_slices.iter().map(|(_,x,_)| x.len()).sum());
        let union = tile_slices.iter().fold(IndexSet::new(), |acc, (_,x,_)| &acc | x);
        assert_eq!(tiles, union);
    }

    #[test]
    fn test_total_para_raster(){
        let points = parallel_read("../0_data_generators/data_100_shuffled.csv").unwrap();
        let precision = 4.;
        let threshold = 5;
        let min_cluster_size = 4;
        let nr_cores = num_cpus::get();
        let (tiles, scalar) = raster::par::map_to_tiles(&points, precision, threshold, nr_cores);
        let slices = raster::par::split_vertically(tiles, -180, 180, scalar, nr_cores);
        let clusters_par = raster::par::cluster_tiles(slices, min_cluster_size);

        // regular raster should give the correct result
        let (tiles_seq, _) = raster::map_to_tiles(&points, precision, threshold);
        let clusters_seq = raster::cluster_tiles(tiles_seq, min_cluster_size);

        assert_eq!(clusters_par.len(), clusters_seq.len());
        for c in clusters_par {
            assert!(clusters_seq.iter().any(|set| *set == c));
        }
    }

    #[test]
    fn test_total_duo_raster(){
        let points = parallel_read("../0_data_generators/data_100_shuffled.csv").unwrap();
        let precision = 4.;
        let threshold = 5;
        let min_cluster_size = 4;
        let nr_cores = num_cpus::get();
        let (left_tiles, right_tiles, _) = raster::par::dual_clustering::map_to_tile_slices(&points, precision, threshold, nr_cores);
        let clusters_par = raster::par::dual_clustering::cluster_tiles(left_tiles, right_tiles, min_cluster_size);

        // regular raster should give the correct result
        let (tiles_seq, _) = raster::map_to_tiles(&points, precision, threshold);
        let clusters_seq = raster::cluster_tiles(tiles_seq, min_cluster_size);

        assert_eq!(clusters_par.len(), clusters_seq.len());
        for c in clusters_par {
            assert!(clusters_seq.iter().any(|set| *set == c));
        }
    }
}