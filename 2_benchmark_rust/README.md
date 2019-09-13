# RASTER implemented in Rust
This is a Cargo package with one bin target and one library.
See usage by running:
```
cargo run -- --help
```
The `--` in the run command above is necessary to provide the arguments to the executable from `cargo run`.


## Generate documentation
```
cargo doc
```
This will generate all documentation (including dependencies).
The RASTER documentation can be found by opening `target/doc/raster/index.html`.


## Running the benchmarks
Before you run any benchmark, make sure that the datasets are available.
In the provided source code, their path is hard-coded as
`../0_data_generators/`, which you may have to change.
The input files were named `data_N_shuffled.csv`, where N was 100, 1000, 10000, 100000, 1000000.

The `--release` flag is used to optimize the program to run faster.

### Sequential RASTER
```
cargo run --release -- --bench
```
### Sequential RASTER'
```
cargo run --release -- --bench --prime
```
### Concurrent RASTER
```
cargo run --release -- par --bench
```
### Concurrent RASTER'
```
cargo run --release -- par --bench --prime
```
