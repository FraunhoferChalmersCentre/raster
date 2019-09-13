/*!
 * The concurrent version of RASTER'.
 *
 * This is very similar to regular RASTER [`par`](../../par/index.html).
 */

use hashbrown::HashMap;
use indexmap::IndexMap;
use rayon::prelude::*;
use std::thread;

use crate::{
    Float,
    Point,
    Tile,
    prime::TileMap,
};


/// Cf. [`par::map_to_tiles`](../../par/fn.map_to_tiles.html)
pub fn map_to_tiles(
    points: &Vec<Point>,
    precision: Float,
    threshold: usize,
    nr_threads: usize,
) -> (impl Iterator<Item=(Tile, Vec<Point>)>, Float) {

    let scalar = (10 as Float).powf(precision);
    let chunk_size = points.len() / nr_threads;

    let accumulate_tiles = points
        .par_chunks(chunk_size)
        .map(|data| {
            let mut all_points = HashMap::new();

            for &p in data {
                let p_int = p.truncate(scalar);

                let count = all_points.entry(p_int).or_insert(vec![]);
                count.push(p);
            }

            all_points
        })
        .reduce_with(|mut acc, tiles| {
            for (t, associated) in tiles {
                let collection = acc.entry(t).or_insert(Vec::new());
                collection.extend(associated);
            }
            acc
        }).expect("Reduce on < 1 map. Is nr_threads set to zero?");

    (
        accumulate_tiles.into_iter()
            .filter(move |(_, count)| count.len() >= threshold ),
        scalar,
    )
}


#[inline]
/// Cf. [`par::split_vertically`](../../par/fn.split_vertically.html)
pub fn split_vertically(
    all_tiles: impl Iterator<Item=(Tile, Vec<Point>)>,
    min: i32,
    max: i32,
    scalar: Float,
    nr_slices: usize
) -> Vec<(i32, TileMap, i32)>{
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
    let mut tile_slices: Vec<(i32, TileMap, i32)> = Vec::with_capacity(splits.len()+1);
    for i in 0..tile_slices.capacity() {
        let left_bound = *splits.get(i.wrapping_sub(1)).unwrap_or(&std::i32::MIN);
        let right_bound = (*splits.get(i).unwrap_or(&std::i32::MIN)).wrapping_sub(1); // split-1 or wrap around to MAX
        tile_slices.push((left_bound, IndexMap::new(), right_bound));
    }
    // Split all_tiles into nr_threads sets
    for (k,v) in all_tiles {
        for i in 0..splits.len() {
            if k.0 < splits[i] {
                tile_slices[i].1.insert(k, v.clone());
                break;
            }
        }
        if k.0 >= *splits.last().unwrap() {
            tile_slices[splits.len()].1.insert(k, v);
        }
    }
    tile_slices
}


/// Cf. [`par::cluster_tiles`](../../par/fn.cluster_tiles.html)
pub fn cluster_tiles(slices: Vec<(i32, TileMap, i32)>, min_cluster_size: usize) -> Vec<TileMap> {

    if slices.len() < 2 {
        if let Some((_, tiles, _)) = slices.into_iter().next() {
            return crate::prime::cluster_tiles(tiles, min_cluster_size);
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
    let clusters_2d: Vec<Vec<TileMap>> = fst;
    let edges: Vec<(Vec<TileMap>, Vec<TileMap>, Vec<TileMap>)> = snd;
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

    // traverse right_edges and left_edges backwards
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


/// Cf. [`par::cluster_slice`]
fn cluster_slice(
    tiles: TileMap,
    min_cluster_size: usize,
    left_edge: i32,
    right_edge: i32
) -> (Vec<TileMap>, (Vec<TileMap>, Vec<TileMap>, Vec<TileMap>)) {
    let mut to_visit = tiles;
    let mut clusters = Vec::new();
    let mut left_clusters = Vec::new();
    let mut right_clusters = Vec::new();
    let mut left_right_clusters = Vec::new();

    while let Some((x, val)) = to_visit.pop() { // starting point for a new cluster
        let mut is_left_edge_cluster = false;
        let mut is_right_edge_cluster = false;
        is_left_edge_cluster |= x.0 == left_edge; // mark cluster if x is an edge-tile
        is_right_edge_cluster |= x.0 == right_edge; // mark cluster if x is an edge-tile

        let mut cluster = IndexMap::new();
        cluster.insert(x, val);

        let mut to_check: Vec<_> = pop_neighbors(x, &mut to_visit).collect();
        while let Some((p, val)) = to_check.pop() {
            is_left_edge_cluster |= p.0 == left_edge; // mark cluster if p is an edge-tile
            is_right_edge_cluster |= p.0 == right_edge; // mark cluster if p is an edge-tile
            cluster.insert(p, val);

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


/// Returns all neighbors to (x,y) in tiles and removes them from tiles.
fn pop_neighbors<'a>((x, y): Tile, tiles: &'a mut TileMap) -> impl Iterator<Item = (Tile, Vec<Point>)>  + 'a {
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

    candidates.into_iter().filter_map( move |n| tiles.remove(&n).map(|v| (n, v)) ) // set's remove returns a bool
}


/// Cf. [`par::join_clusters`]
fn join_clusters(
    left_clusters: Vec<TileMap>,
    right_clusters: Vec<TileMap>,
    left_right_clusters: Vec<TileMap>,
    min_cluster_size: usize,
) -> (Vec<TileMap>, Vec<TileMap>,) {
    let mut clusters = Vec::new();
    let mut new_left_right_clusters = Vec::new();
    let mut left_right = left_right_clusters;
    let mut left = left_clusters;
    let mut right = right_clusters;
    let mut is_left_and_right;

    while let Some(start) = right.pop() {
        is_left_and_right = false;
        let mut to_visit = vec![(false, start)];
        let mut cluster = IndexMap::new();

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


fn is_cluster_neighbors(c1: &TileMap, c2: &TileMap) -> bool {
    for (tile, _) in c1.iter() {
        if is_neighbors(*tile, c2) {
            return true;
        }
    }
    false
}


fn is_neighbors((x, y): Tile, tiles: &TileMap) -> bool {
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

    candidates.iter().any(|tile| tiles.contains_key(tile))
}
