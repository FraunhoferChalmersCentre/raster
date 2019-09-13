mod data;
#[macro_use] // timeit!
mod benchmark_lib;

use benchmark_lib::{
    cluster_iter,
    Raster,
    write_bench_times,
};
use raster::Float;
use std::fs::remove_file;
use docopt::Docopt;
use serde::Deserialize;



const USAGE: &'static str = "
RASTER is an approximation algorithm for clustering.
It can either run sequentially or concurrently. There is also a prime version of each.

Usage: raster [options] [[-p P]... [-n N]... | --bench]
       raster par [options] [[-p P]... [-n N]... <cores>... | --bench]
       raster [-h | --help]

Command:
    par  Run concurrent RASTER with one or more number of threads/cores.

Options:
    -h, --help     Show this message.
    -p P           Precision makes tiles by keeping P decimal digits [default: 3.5].
    -t T           Threshold for significant tiles [default: 5].
    -m M           Minimum number of tiles in a cluster [default: 4]
    -i N           #Iterations to run the benchmark [default: 5].
    -n <clusters>  The number of clusters [default: 1000].
    --prime        Use RASTER' where the points are retained.
    --bench        Sets precision, #clusters, and #cores depending on <type>.
                   When a sequential type:
                     precision = 3, 3.5, 4, 5
                     #clusters = 10^2, 10^3, 10^4, 10^5, 10^6
                   When a parallel type:
                     precision = 3, 3.5, 4
                     #clusters = 10^5, 10^6
                     #cores    = 1, 2, 4, 8
";


#[derive(Deserialize, Debug)]
struct Args {
    cmd_par: bool,
    arg_cores: Vec<usize>,
    flag_help: bool,
    flag_p: Vec<Float>, // precision [p]
    flag_t: usize, // threshold
    flag_m: usize, // min cluster size
    flag_i: usize, // iterations
    flag_n: Vec<usize>, // [n] clusters
    flag_prime: bool,
    flag_bench: bool,
}


fn main() {
    let args: Args = Docopt::new(USAGE)
        .map(|d| d.help(true))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let alg = match (args.cmd_par, args.flag_prime) {
        (false, false) => Raster::Seq,
        (false, true)  => Raster::SeqPrime,
        (true, false)  => Raster::Par,
        (true, true)   => Raster::ParPrime,
    };
    let mut nr_clusters_list = args.flag_n;
    let mut nr_cores = if args.cmd_par { args.arg_cores } else { vec![1] };
    let mut precisions = args.flag_p;
    let threshold = args.flag_t;
    let min_size = args.flag_m;
    let iterations = args.flag_i;

    if args.flag_bench {
        match alg {
            Raster::Seq | Raster::SeqPrime => {
                nr_clusters_list = vec![100, 1_000, 10_000, 100_000, 1_000_000];
                precisions = vec![3.0, 3.5, 4.0, 5.0];
            },
            Raster::Par | Raster::ParPrime => {
                nr_clusters_list = vec![100_000, 1_000_000];
                precisions = vec![3.0, 3.5, 4.0];
                nr_cores = vec![1, 2, 4, 8];
            },
        }
    }

    let alg_str = match alg {
        Raster::Seq       => "raster",
        Raster::SeqPrime  => "raster_prime",
        Raster::Par       => "par_raster",
        Raster::ParPrime  => "par_raster_prime",
    };

    for precision in precisions.iter() {
        let _ = remove_file(format!("output/{}_rust_precision{}_threshold{}_min_size{}.csv"
                                    , alg_str, precision, threshold, min_size));
    }

    for &nr_clusters in nr_clusters_list.iter() {
        let file = format!("../0_data_generators/data_{}_shuffled.csv", nr_clusters);
        println!("{} clusters", nr_clusters);
        let (points, _time) = timeit!("Reading: {} ms",
            data::parallel_read(file).unwrap()
        );
        println!("I found {} data points.", points.len());

        for &cores in nr_cores.iter() {
            for &precision in precisions.iter() {
                let row = cluster_iter(&alg, &points, nr_clusters, precision, threshold, cores, min_size, iterations);

                let desc_file = format!("output/{}_rust_precision{}_threshold{}_min_size{}.csv"
                                    , alg_str, precision, threshold, min_size);
                write_bench_times(row, desc_file).unwrap();
            }
        }
    }
}
