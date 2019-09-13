
use std::error::Error;
use std::fs::{create_dir, OpenOptions};
use std::path::Path;
use raster::Float;
use raster::Point;


macro_rules! timeit {
    // Returns micro seconds [µs] and prints [ms]
    ($format:tt, $code:expr) => ({
        use std::time::Instant;
        let time = Instant::now();
        let retur = $code;
        let elap = time.elapsed();
        println!($format, elap.as_micros() as f64 / 1_000.);
        (retur, elap.as_micros())
    });
}


pub struct CsvFormat {
    pub mean: f64,
    pub std_dev: f64,
    pub proj_mean: f64,
    pub proj_std_dev: f64,
    pub proj_times: Vec<f64>,
    pub clust_mean: f64,
    pub clust_std_dev: f64,
    pub clust_times: Vec<f64>,
    pub nr_clusters: usize,
    pub nr_clusters_percent: f32,
    pub nr_cores: usize,
}

impl CsvFormat {
    pub fn mk_record(&self) -> Vec<String> {
        vec![
            self.nr_clusters.to_string(),
            self.nr_clusters_percent.to_string(),
            self.nr_cores.to_string(),
            self.mean.to_string(),
            self.std_dev.to_string(),
            self.proj_mean.to_string(),
            self.clust_mean.to_string(),
            self.proj_std_dev.to_string(),
            self.clust_std_dev.to_string(),
            format!("{:?}", self.proj_times).replace("\"", ""),
            format!("{:?}", self.clust_times).replace("\"", ""),
        ]
    }

    pub fn header() -> Vec<&'static str> {
        vec![
            "#clusters",
            "#clusters identified [ratio]",
            "#cores",
            "mean [s]",
            "sample std_dev [s]",
            "mean projection [s]",
            "mean clustering [s]",
            "sample std_dev projection [s]",
            "sample std_dev clustering [s]",
            "times projection [s]",
            "times clustering [s]",
        ]
    }
}


#[derive(Debug)]
pub enum Raster {
    Seq,
    SeqPrime,
    Par,
    ParPrime
}


/// Run multiple iterations of RASTER and return a benchmark summary.
pub fn cluster_iter(
    algorithm_choice: &Raster,
    points: &Vec<Point>,
    nr_clusters: usize,
    precision: Float,
    threshold: usize,
    nr_cores: usize,
    min_size: usize,
    iterations: usize,
) -> CsvFormat {
    println!("------------------------------\nRunning: {:?}, p={}, #cores={}\n",
        algorithm_choice, precision, nr_cores);

    let mut proj_secs = Vec::new();
    let mut clust_secs = Vec::new();
    let mut identifications = Vec::new();

    for _ in 0..iterations {
        let (proj_ms, clust_ms, n_clusters_ident) = match algorithm_choice {
            Raster::Seq      => seq_raster(&points, precision, threshold, min_size),
            Raster::SeqPrime => seq_raster_prime(&points, precision, threshold, min_size),
            Raster::Par      => par_raster(points, precision, threshold, nr_cores, min_size),
            Raster::ParPrime => par_raster_prime(points, precision, threshold, nr_cores, min_size),
        };
        proj_secs.push(proj_ms as f64 / 1_000_000.);
        clust_secs.push(clust_ms as f64 / 1_000_000.);
        identifications.push(n_clusters_ident);
    }

    let proj_avg = mean(&proj_secs);
    let proj_std_dev = std_dev(&proj_secs, proj_avg);
    let clust_avg = mean(&clust_secs);
    let clust_std_dev = std_dev(&clust_secs, clust_avg);
    println!("\nAverage time:\n\tprojection: {:.4} s\n\tclustering: {:.4} s",
        proj_avg, clust_avg);
    println!("Standard deviation:\n\tprojection: {:.4} s\n\tclustering: {:.4} s",
        proj_std_dev, clust_std_dev);

    let avg = proj_avg + clust_avg;
    let secs = proj_secs.iter().zip(&clust_secs).map(|(x, y)| x+y).collect();
    let sd = std_dev(&secs, avg);
    println!("Total average time: {:.3} s\n", avg);

    let n_clusters_ident = identifications.pop().unwrap();
    assert!(identifications.into_iter().all(|n| n == n_clusters_ident));

    CsvFormat {
        mean:                avg,
        std_dev:             sd,
        proj_mean:           proj_avg,
        proj_std_dev:        proj_std_dev,
        proj_times:          proj_secs,
        clust_mean:          clust_avg,
        clust_std_dev:       clust_std_dev,
        clust_times:         clust_secs,
        nr_clusters:         nr_clusters,
        nr_clusters_percent: n_clusters_ident as f32 / nr_clusters as f32,
        nr_cores:            nr_cores,
    }
}


fn seq_raster(points: &Vec<Point>, precision: Float, threshold: usize, min_cluster_size: usize) -> (u128, u128, usize) {
    let ((tiles, _scalar), proj_microsec) = timeit!("Projection: {} ms",
        raster::map_to_tiles(points, precision, threshold)
    );
    let (clusters, clust_microsec) = timeit!("Clustering: {} ms",
        raster::cluster_tiles(tiles, min_cluster_size)
    );
    (proj_microsec, clust_microsec, clusters.len())
}


fn seq_raster_prime(points: &Vec<Point>, precision: Float, threshold: usize, min_cluster_size: usize) -> (u128, u128, usize) {
    let ((tiles, _scalar), proj_microsec) = timeit!("Projection: {} ms",
        raster::prime::map_to_tiles(points, precision, threshold as usize)
    );
    let (clusters, clust_microsec) = timeit!("Clustering: {} ms",
        raster::prime::cluster_tiles(tiles, min_cluster_size)
    );
    (proj_microsec, clust_microsec, clusters.len())
}


fn par_raster(points: &Vec<Point>, precision: Float, threshold: usize, nr_cores:usize, min_cluster_size: usize) -> (u128, u128, usize) {
    let ((slices, _scalar), proj_microsec) = timeit!("Projection: {} ms",{
        let (tiles, scalar) = raster::par::map_to_tiles(points, precision, threshold, nr_cores);
        let slices = raster::par::split_vertically(tiles, -180, 180, scalar, nr_cores);
        (slices, scalar)
    });
    let (clusters, clust_microsec) = timeit!("Clustering: {} ms",
        raster::par::cluster_tiles(slices, min_cluster_size)
    );
    (proj_microsec, clust_microsec, clusters.len())
}


fn par_raster_prime(points: &Vec<Point>, precision: Float, threshold: usize, nr_cores:usize, min_cluster_size: usize) -> (u128, u128, usize) {
    let ((slices, _scalar), proj_microsec) = timeit!("Projection: {} ms",{
        let (tiles, scalar) = raster::prime::par::map_to_tiles(points, precision, threshold, nr_cores);
        let slices = raster::prime::par::split_vertically(tiles, -180, 180, scalar, nr_cores);
        (slices, scalar)
    });
    let (clusters, clust_microsec) = timeit!("Clustering: {} ms",
        raster::prime::par::cluster_tiles(slices, min_cluster_size)
    );
    (proj_microsec, clust_microsec, clusters.len())
}


pub fn write_bench_times<P: AsRef<Path>>(
    csv_row: CsvFormat,
    path: P,
) -> Result<(), Box<Error>> {
    if let Some(dir) = path.as_ref().parent() {
        let _ = create_dir(dir);
    }
    let add_header = !path.as_ref().exists();
    let writer = OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path)?;
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b';')
        .from_writer(writer);

    if add_header {
        wtr.write_record(CsvFormat::header())?;
    }
    wtr.write_record(csv_row.mk_record())?;
    wtr.flush()?;
    Ok(())
}


fn mean(numbers: &Vec<f64>) -> f64 {
    let sum: f64 = numbers.iter().sum();
    sum as f64 / numbers.len() as f64
}

/// sample standard deviation:
/// √(Σ(x-µ)²/(n-1))
fn std_dev(numbers: &Vec<f64>, mean: f64) -> f64 {
    let sum: f64 = numbers.iter().map(|x| (*x as f64 - mean).powi(2)).sum();
    (sum / (numbers.len() - 1) as f64).sqrt()
}

#[test]
fn std_test() {
    let input = vec![9., 2., 5., 4.];
    let mean = mean(&input);
    let std = std_dev(&input, mean);
    assert_eq!(std, (26. / 3f64).sqrt())
}