/*!
 * This is the implementation of Contraction Clustering (RASTER) for 2D data.
 * It is covered in detail in a [paper](https://doi.org/10.1007/978-3-319-72926-8_6).
 */

pub mod par;
pub mod prime;

use indexmap::IndexSet;
use hashbrown::HashMap;
use serde::Deserialize;


pub type Tile = (i32, i32);
pub type TileSet = IndexSet<Tile>;
pub type Float = f64;
#[derive(Debug, Deserialize, PartialEq, Clone, Copy)]
pub struct Point(Float, Float);

impl Point {
    pub fn new(x: Float, y: Float) -> Self {
        Point(x, y)
    }

    pub fn truncate(&self, scalar: Float) -> Tile{
        ( (self.0 * scalar) as i32, (self.1 * scalar) as i32 )
    }
}


/// Counts the number of points for each tile containing at least one point.
fn to_tile_counts<'a>(
    points: &'a [Point],
    scalar: Float,
) -> HashMap<Tile, usize> {
    let mut tile_count = HashMap::new();

    for p in points {
        let p_int = p.truncate(scalar);
        let count = tile_count.entry(p_int).or_insert(0);
        *count += 1;
    }

    tile_count
}


/// Converts points into a set of significant tiles.
/// A tile is significant if it maps to at least `threshold` number of points.
/// Also returns the scaling factor used for creating tiles.
pub fn map_to_tiles(
    points: &Vec<Point>,
    precision: Float,
    threshold: usize,
) -> (TileSet, Float) {
    let scalar = (10 as Float).powf(precision);
    let all_tiles = to_tile_counts(points, scalar);

    let tiles = all_tiles
        .into_iter()
        .filter(|(_, count)| count >= &threshold )
        .map(|(tile, _)| tile)
        .collect();

    (tiles, scalar)
}


/// Cluster a set of significant tiles.
pub fn cluster_tiles(tiles: TileSet, min_cluster_size: usize) -> Vec<TileSet> {
    let mut to_visit = tiles;
    let mut clusters = Vec::new();

    while let Some(x) = to_visit.pop() { // starting point for a new cluster
        let mut cluster = IndexSet::new(); // NOTE: if we don't need sets it faster with a Vec
        cluster.insert(x);

        let mut to_check: Vec<Tile> = pop_neighbors(x, &mut to_visit).collect();
        while let Some(p) = to_check.pop() {
            cluster.insert(p);

            let new_neighbors = pop_neighbors(p, &mut to_visit);
            to_check.extend(new_neighbors);
        }

        if cluster.len() >= min_cluster_size {
            clusters.push(cluster);
        }
    }
    clusters
}


/// Returns all neighbors to (x,y) in tiles and removes them from tiles.
fn pop_neighbors<'a>((x, y): Tile, tiles: &'a mut TileSet) -> impl Iterator<Item = Tile>  + 'a {
    let candidates = vec![
        (x + 1, y    ),
        (x - 1, y    ),
        (x    , y + 1),
        (x    , y - 1),
        (x + 1, y - 1),
        (x + 1, y + 1),
        (x - 1, y - 1),
        (x - 1, y + 1),
    ];

    candidates.into_iter().filter( move |n| tiles.remove(n) ) // set's remove returns a bool
}



/////////////////////////////////////////////////////////////////////////////////////
/// Unit tests
/////////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping() {
        let points = vec![Point(1., 23.22), Point(1.05, 23.28)];
        let scalar = 10.;
        // Fixed sized set:
        let truth = [ (10, 232) ].iter().cloned().collect();

        assert_eq!(map_to_tiles(&points, 1., 2), (truth, scalar));
    }

    #[test]
    fn mapping_to_empty() {
        let points = vec![Point(1., 23.22), Point(1.05, 23.28)];
        let scalar = 10.;
        let truth = IndexSet::new();

        assert_eq!(map_to_tiles(&points, 1., 3), (truth, scalar));
    }

    #[test]
    fn who_are_my_neighbors() {
        let tile = (2, 5);
        let mut tiles =  [
            (3, 5),
            (0, 5),
            (2, 4),
            (1, 4),
        ].iter().cloned().collect();

        let truth = vec![
            (3, 5),
            (2, 4),
            (1, 4),
        ];

        let result: Vec<_> = pop_neighbors(tile, &mut tiles).collect();
        assert_eq!(result, truth);
    }

    #[test]
    fn no_neighbors() {
        let tile = (2, 5);
        let mut tiles = IndexSet::new();

        let result: Vec<_> = pop_neighbors(tile, &mut tiles).collect();
        assert_eq!(result, Vec::new());
    }

    #[test]
    fn clustering() {

        let input = [
            (0, 0),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (5, 0),
            (5, 1),
            (5, 2),
        ].iter().cloned().collect();

        let output: Vec<TileSet> = vec![
            [
                (0 ,  0),
                (-1,  0),
                (-1, -1),
                (0 , -1),
            ].iter().cloned().collect(),
            [
                (5, 0),
                (5, 1),
                (5, 2),
            ].iter().cloned().collect(),
        ];

        let res = cluster_tiles(input, 1);

        // This is a way of considering if they are equal without
        // taking the order of the clusters into consideration.
        assert_eq!(res.len(), output.len());
        for o in output {
            assert!(res.iter().any(|set| *set == o));
        }
    }

    #[test]
    fn map_to_tile_slices2(){
        let points = vec![
            Point(-100., 23.22),
            Point(-1., 23.28),
            Point(50., 23.28),
            Point(100., 23.28),
        ];
        let precision = 5.;
        let threshold = 1;
        let (tiles, scalar) = par::map_to_tiles(&points, precision, threshold, 4);
        let tile_slices = par::split_vertically(tiles, -180, 180, scalar, 4);
        // regular mapping should give the correct result
        let (tiles, _) = map_to_tiles(&points, precision, threshold);

        //let intersect = tile_slices.iter().fold(IndexSet::new(), |acc, x| &acc & x);
        //assert_eq!(intersect, IndexSet::new());

        assert_eq!(tiles.len(), tile_slices.iter().map(|(_,x,_)| x.len()).sum());
        let union = tile_slices.iter().fold(IndexSet::new(), |acc, (_,x,_)| &acc | x);
        assert_eq!(tiles, union);
    }

    #[test]
    fn left_right_edge_case() {
        let slices = vec![
            (std::i32::MIN, [(-2,0)].iter().cloned().collect(), -2),
            (-1, [(-1,0), (0,0), (0,-2)].iter().cloned().collect(), 0),
            (1, [(1,1), (2,1), (1,-2)].iter().cloned().collect(), 2),
            (3, [].iter().cloned().collect(), std::i32::MAX),
        ];
        let clusters2 = par::cluster_tiles(slices, 2);
        let regular_input = [(-2,0), (-1,0), (0,0), (0,-2), (1,1), (2,1), (1,-2)].iter().cloned().collect();
        let clusters1 = cluster_tiles(regular_input, 2);

        assert_eq!(clusters1.len(), clusters2.len());
        for c in clusters2 {
            assert!(clusters1.iter().any(|set| *set == c));
        }
    }
}
