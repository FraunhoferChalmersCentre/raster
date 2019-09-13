/*!
 * The concurrent version of RASTER.
 *
 * There are a few differences compared to the sequential interface:
 * - extra parameters (e.g. `nr_threads`)
 * - [`map_to_tiles`](fn.map_to_tiles.html) returns an iterator instead of a set
 * - [`split_vertically`](fn.split_vertically.html) is a preprocessing step for [`cluster_tiles`](fn.cluster_tiles.html)
 */

use indexmap::IndexSet;
use hashbrown::HashMap;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;

pub mod dual_clustering;

use crate::{
    Float,
    Point,
    Tile,
    TileSet,
    pop_neighbors,
    to_tile_counts,
};


/// Split data into `nr_parts` batches.
fn batch_data<'a>(points: &'a Vec<Point>, nr_parts: usize) -> impl Iterator<Item = Vec<Point>> + 'a {
    let chunk_size = points.len() / nr_parts;
    points.chunks(chunk_size).map(|b| b.to_vec())
}


/// Concurrent version of [`map_to_tiles`](../fn.map_to_tiles.html).
pub fn map_to_tiles(
    points: &Vec<Point>,
    precision: Float,
    threshold: usize,
    nr_threads: usize,
) -> (impl Iterator<Item=Tile>, Float) {

    type Counter = HashMap<Tile, usize>;
    let (sx, rx): (Sender<Counter>, Receiver<Counter>) = mpsc::channel();
    let scalar = (10 as Float).powf(precision);
    let batches = batch_data(points, nr_threads);

    for data in batches {
        let thread_sx = sx.clone();
        thread::spawn(move || {
            let tiles = to_tile_counts(&data, scalar);
            thread_sx.send(tiles).unwrap();
        });
    }
    drop(sx); // need to drop all Sender references

    let mut accumulate_tiles = rx.recv().unwrap();
    if nr_threads > 1 {
        accumulate_tiles.reserve(accumulate_tiles.len()) // add more capacity
    };
    // Wait for the threads to finish and sum counters for each tile.
    while let Ok(tiles) = rx.recv() {
        for (t, t_count) in tiles {
            let count = accumulate_tiles.entry(t).or_insert(0);
            *count += t_count;
        }
    }

    (
        accumulate_tiles.into_iter()
            .filter(move |(_, count)| *count >= threshold )
            .map(|(tile, _)| tile),
        scalar,
    )
}


/// Takes an iterator of significant tiles and split them into `nr_slices` slices
/// depending on their spacial horizontal alignments. These tiles need to be between
/// `min` and `max` on the y-axis and this bound will be scaled with `scalar` to fit the tiles.
#[inline]
pub fn split_vertically(
    all_tiles: impl Iterator<Item=Tile>,
    min: i32,
    max: i32,
    scalar: Float,
    nr_slices: usize,
) -> Vec<(i32, TileSet, i32)>{
    // Compute splitting coordinates depending on nr_slices.
    if nr_slices < 2 {
        return vec![(std::i32::MIN, all_tiles.collect(), std::i32::MAX)];
    }

    let step = ((max.abs() + min.abs()) as Float / nr_slices as Float) as i32;
    let mut splits = Vec::new();
    let mut split = min + step;
    for _ in 1..nr_slices {
        splits.push((split as Float * scalar) as i32); // scale and push
        split += step;
    }

    // Initialize tile_slices with border values
    let mut tile_slices: Vec<(i32, TileSet, i32)> = Vec::with_capacity(splits.len()+1);
    for i in 0..tile_slices.capacity() {
        let left_bound = *splits.get(i.wrapping_sub(1)).unwrap_or(&std::i32::MIN);
        let right_bound = (*splits.get(i).unwrap_or(&std::i32::MIN)).wrapping_sub(1); // split-1 or wrap around to MAX
        tile_slices.push((left_bound, IndexSet::new(), right_bound));
    }
    // Split all_tiles into nr_threads sets
    for x in all_tiles {
        for i in 0..splits.len() {
            if x.0 < splits[i] {
                tile_slices[i].1.insert(x);
                break;
            }
        }
        if x.0 >= *splits.last().unwrap() {
            tile_slices[splits.len()].1.insert(x);
        }
    }
    tile_slices
}


/// Concurrent version of [`cluster_tiles`](../fn.cluster_tiles.html).
pub fn cluster_tiles(slices: Vec<(i32, TileSet, i32)>, min_cluster_size: usize) -> Vec<TileSet> {

    if slices.len() < 2 {
        if let Some((_, tiles, _)) = slices.into_iter().next() {
            return crate::cluster_tiles(tiles, min_cluster_size);
        }
        else {
            return vec![];
        }
    }

    let mut forks = Vec::new();
    for (left, tiles, right) in slices {
        forks.push(thread::spawn(move || {
            cluster_slice(tiles, min_cluster_size, left, right)
        }));
    }

    let (fst, snd) = forks.into_iter().map(|f| f.join().unwrap()).unzip();
    let clusters_2d: Vec<Vec<TileSet>> = fst;
    let edges: Vec<(Vec<TileSet>, Vec<TileSet>, Vec<TileSet>)> = snd;
    let mut left_edges = Vec::new();
    let mut left_right_edges = Vec::new();
    let mut right_edges = Vec::new();
    for (left, both, right) in edges {
        left_edges.push(left);
        left_right_edges.push(both);
        right_edges.push(right);
    }

    assert_eq!(left_edges.len(), right_edges.len());
    assert_eq!(left_edges.len(), left_right_edges.len());

    let mut clusters = Vec::new();
    clusters_2d.into_iter().for_each(|c|
        clusters.extend(c)
    );

    // traverse right_edges, left_edges, and left_right_edges backwards (from right to left)
    assert_eq!(0, right_edges.pop().unwrap().len()); // The right most is touching infinity (i32::MAX),
                                                     // which no cluster should be able to do.
    assert_eq!(0, left_right_edges.pop().unwrap().len());

    let mut trans_slices = Vec::new();
    while let Some(right) = right_edges.pop() {
        let mut left = left_edges.pop().unwrap();
        left.extend(trans_slices);
        let left_right = left_right_edges.pop().unwrap();
        // Note that "right" and "left" switch place in the call to join_clusters.
        // This is because "right" is clusters to the right within a slice while the parameter
        // to join_clusters refers to the right side of a border between slices
        let (joined_clusters, transient) = join_clusters(right, left, left_right, min_cluster_size);
        trans_slices = transient;
        clusters.extend(joined_clusters);
    }

    clusters.extend(trans_slices.into_iter().filter(|c| c.len() >= min_cluster_size));

    assert_eq!(0, left_edges.pop().unwrap().len()); // The right most is touching infinity (i32::MIN),
                                                    // which no cluster should be able to do.

    clusters
}


/// Regular clustering within a slice, where clusters that touch left or right edge are marked in
/// the output: (Significant clusters, (left clusters, left & right clusters, right clusters)).
/// Significant clusters are those that does not touch any edge and contains at least
/// `min_cluster_size` tiles.
fn cluster_slice(
    tiles: TileSet,
    min_cluster_size: usize,
    left_edge: i32,
    right_edge: i32,
) -> (Vec<TileSet>, (Vec<TileSet>, Vec<TileSet>, Vec<TileSet>)) {
    let mut to_visit = tiles;
    let mut clusters = Vec::new();
    let mut left_clusters = Vec::new();
    let mut right_clusters = Vec::new();
    let mut left_right_clusters = Vec::new();

    while let Some(x) = to_visit.pop() { // starting point for a new cluster
        let mut is_left_edge_cluster = false;
        let mut is_right_edge_cluster = false;
        is_left_edge_cluster |= x.0 == left_edge; // mark cluster if x is an edge-tile
        is_right_edge_cluster |= x.0 == right_edge; // mark cluster if x is an edge-tile

        let mut cluster = IndexSet::new();
        cluster.insert(x);

        let mut to_check: Vec<_> = pop_neighbors(x, &mut to_visit).collect();
        while let Some(p) = to_check.pop() {
            is_left_edge_cluster |= p.0 == left_edge; // mark cluster if p is an edge-tile
            is_right_edge_cluster |= p.0 == right_edge; // mark cluster if p is an edge-tile
            cluster.insert(p);

            let new_neighbors = pop_neighbors(p, &mut to_visit);
            to_check.extend(new_neighbors);
        }

        if is_left_edge_cluster && is_right_edge_cluster {
            left_right_clusters.push(cluster);
        }
        else if is_left_edge_cluster {
            left_clusters.push(cluster);
        }
        else if is_right_edge_cluster {
            right_clusters.push(cluster);
        }
        else if cluster.len() >= min_cluster_size {
            clusters.push(cluster);
        }
    }
    (clusters, (left_clusters, left_right_clusters, right_clusters))
}


/// Join clusters that are part of a neighborhood chain (connected). The left_clusters are those
/// that touches the border from the left side, right_clusters from the right side and
/// left_right_clusters are those that touches the border from the left side and another border to
/// the left of this one.
/// The output is two things:
/// * significant clusters (#tiles >= min_cluster_size)
/// * clusters that are a superset of a left_right_clusters
/// These clusters were the result of joining multiple clusters if there were a connection.
fn join_clusters(
    left_clusters: Vec<TileSet>,
    right_clusters: Vec<TileSet>,
    left_right_clusters: Vec<TileSet>,
    min_cluster_size: usize,
) -> (Vec<TileSet>, Vec<TileSet>,) {
    let mut clusters = Vec::new();
    let mut new_left_right_clusters = Vec::new();
    let mut left_right = left_right_clusters;
    let mut left = left_clusters;
    let mut right = right_clusters;
    let mut is_left_and_right;

    while let Some(start) = right.pop() {
        is_left_and_right = false;
        let mut to_visit = vec![(false, start)];
        let mut cluster = IndexSet::new();

        while let Some((go_right, visit)) = to_visit.pop() {
            if go_right {
                let (neighbors, leftovers) = right.into_iter().partition(|c| is_cluster_neighbors(c, &visit));
                right = leftovers;

                for n in neighbors {
                    to_visit.push((false, n));
                }
            } else {
                let (neighbors, leftovers) = left.into_iter().partition(|c| is_cluster_neighbors(c, &visit));
                left = leftovers;

                for n in neighbors {
                    to_visit.push((true, n));
                }
                // when going from right to left left_right_clusters is considered to be on the left side
                let (neighbors, leftovers) = left_right.into_iter().partition(|c| is_cluster_neighbors(c, &visit));
                left_right = leftovers; // remove old (unjoined) cluster

                for n in neighbors {
                    is_left_and_right = true; // assumed to happen quite rarely
                    to_visit.push((true, n));
                }
            }
            cluster.extend(visit); // join the visited sub-cluster
        }
        if is_left_and_right {
            new_left_right_clusters.push(cluster) // insert new (joined) cluster
        }
        else if cluster.len() >= min_cluster_size {
            clusters.push(cluster);
        }
    }
    for c in left {
        if c.len() >= min_cluster_size {
            clusters.push(c);
        }
    }
    left_right.extend(new_left_right_clusters);
    (clusters, left_right)
}


/// Returns true if cluster c1 is a neighbor to cluster c2.
fn is_cluster_neighbors(c1: &TileSet, c2: &TileSet) -> bool {
    for tile in c1.iter() {
        if is_neighbors(*tile, c2) {
            return true;
        }
    }
    false
}


/// Returns true if tile (x,y) is a neighbor to a tile in a set.
fn is_neighbors((x, y): Tile, tiles: &TileSet) -> bool {
    let candidates = [
        (x + 1, y    ),
        (x - 1, y    ),
        (x    , y + 1),
        (x    , y - 1),
        (x + 1, y - 1),
        (x + 1, y + 1),
        (x - 1, y - 1),
        (x - 1, y + 1),
    ];

    candidates.iter().any(|c| tiles.contains(c))
}



#[test]
fn join_right_to_left_n_right_to_right() {
    use std::iter::FromIterator;

    let left: Vec<TileSet> = vec![];
    let left_right: Vec<TileSet> = vec![IndexSet::from_iter(vec![(-2, 1), (-1, 1)])];
    let right: Vec<TileSet> = vec![
        IndexSet::from_iter(vec![(0, 0)]),
        IndexSet::from_iter(vec![(0, 2)]),
    ];
    let (lr, rlrr) = join_clusters(left, right, left_right, 2);

    let long_cluster: TileSet = IndexSet::from_iter(vec![(-2, 1), (-1, 1), (0, 0), (0, 2)]);

    assert_eq!(lr.len(), 0);

    assert_eq!(rlrr.len(), 1);
    assert_eq!(long_cluster.len(), rlrr[0].len());
    for o in rlrr[0].clone() {
        assert!(long_cluster.iter().any(|set| *set == o));
    }
}
