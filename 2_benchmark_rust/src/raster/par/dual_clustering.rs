/*!
 * Initial prototype where cluster_tiles has a fixed parallelism of 2.
 * This code is a bit easier to follow than the general implementation.
 */

use indexmap::IndexSet;
use std::thread;

use crate::{
    Float,
    Point,
    TileSet,
    par::batch_data,
    pop_neighbors,
    par::is_cluster_neighbors,
};
use crate::to_tile_counts;



pub fn map_to_tile_slices(
    points: &Vec<Point>,
    precision: Float,
    threshold: usize,
    nr_threads: usize,
) -> (TileSet, TileSet, Float) {
    let mut workers = vec![];

    let scalar = (10 as Float).powf(precision);
    let batches = batch_data(points, nr_threads);
    for data in batches {
        // Spin up another thread
        workers.push(thread::spawn(move || {
            to_tile_counts(&data, scalar)
        }));
    }

    let mut accumulate_tiles = workers.pop().unwrap().join().unwrap();
    // Wait for the threads to finish and sum counters for each tile.
    for work in workers {
        let tiles = work.join().unwrap();

        for (t, t_count) in tiles {
            let count = accumulate_tiles.entry(t).or_insert(0);
            *count += t_count;
        }
    }

    let (tiles_left, tiles_right) = accumulate_tiles
        .into_iter()
        .filter(|(_, count)| count >= &threshold )
        .map(|(tile, _)| tile)
        .partition(|(x,_y)| *x < 0);

    (
        tiles_left,
        tiles_right,
        scalar,
    )
}


pub fn cluster_tiles(left_tiles: TileSet, right_tiles: TileSet, min_cluster_size: usize) -> Vec<TileSet> {

    let fork1 = thread::spawn(move || {
        cluster_slice(left_tiles, min_cluster_size, Edge::Left)
    });
    let fork2 = thread::spawn(move || {
        cluster_slice(right_tiles, min_cluster_size, Edge::Right)
    });
    let (left_clusters, left_join) = fork1.join().unwrap();
    let (right_clusters, right_join) = fork2.join().unwrap();

    let mid_clusters = join_clusters(left_join, right_join, min_cluster_size);

    let mut left_clusters = left_clusters;
    left_clusters.extend(mid_clusters);
    left_clusters.extend(right_clusters);
    left_clusters
}

fn join_clusters(left_clusters: Vec<TileSet>, right_clusters: Vec<TileSet>, min_cluster_size: usize) -> Vec<TileSet> {
    let mut clusters = Vec::new();
    let mut xs = left_clusters;
    let mut ys = right_clusters;

    while let Some(start) = xs.pop() {
        let mut to_visit = vec![(true, start.clone())];
        let mut cluster = start;

        while let Some((go_right, x)) = to_visit.pop() {
            if go_right {
                let (neighbors, ys_left) = ys.into_iter().partition(|c| is_cluster_neighbors(c, &x));
                ys = ys_left;

                for n in neighbors {
                    to_visit.push((false, n.clone()));
                    cluster.extend(n);
                }
            } else {
                let (neighbors, xs_left) = xs.into_iter().partition(|c| is_cluster_neighbors(c, &x));
                xs = xs_left;

                for n in neighbors {
                    to_visit.push((true, n.clone()));
                    cluster.extend(n);
                }
            }
        }
        if cluster.len() >= min_cluster_size {
            clusters.push(cluster);
        }
    }
    for y in ys {
        if y.len() >= min_cluster_size {
            clusters.push(y);
        }
    }
    clusters
}

#[derive(Copy, Clone)]
enum Edge {
    Left = -1,
    Right = 0,
}

fn cluster_slice(tiles: TileSet, min_cluster_size: usize, y_edge: Edge) -> (Vec<TileSet>, Vec<TileSet>) {
    let mut to_visit = tiles;
    let mut clusters = Vec::new();
    let mut edge_clusters = Vec::new();
    let edge = y_edge as i32;

    while let Some(x) = to_visit.pop() { // starting point for a new cluster
        let mut is_edge_cluster = false;
        is_edge_cluster |= x.0 == edge; // mark cluster if x is an edge-tile

        let mut cluster = IndexSet::new();
        cluster.insert(x);

        let mut to_check: Vec<_> = pop_neighbors(x, &mut to_visit).collect();
        while let Some(p) = to_check.pop() {
            is_edge_cluster |= p.0 == edge; // mark cluster if p is an edge-tile
            cluster.insert(p);

            let new_neighbors = pop_neighbors(p, &mut to_visit);
            for n in new_neighbors {
                to_check.push(n);
            }
        }

        if is_edge_cluster {
            edge_clusters.push(cluster);
        }
        else if cluster.len() >= min_cluster_size {
            clusters.push(cluster);
        }
    }
    (clusters, edge_clusters)
}


/////////////////////////////////////////////////////////////////////////////////////
/// Unit tests
/////////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zip_the_clusters() {
        let xs = vec![
            vec![(-1, 4), (-1, 3)],
            vec![(-1, -1)],
            vec![(-1, -3)],
            vec![(-1, -5)],
        ].into_iter().map(|list| list.iter().cloned().collect()).collect();
        let ys = vec![
            vec![(0, 5), (0, 4)],
            vec![(0, 1), (1, 1), (1, 0), (1, -1)],
            vec![(0, -3), (0, -4), (0, -5)],
        ].into_iter().map(|list| list.iter().cloned().collect()).collect();

        let new_cluster = join_clusters(xs, ys, 4);
        assert_eq!(new_cluster.len(), 3);
    }

    #[test]
    fn zip_the_clusters2() {
        let xs = vec![
            [(-1, 2680001), (-1, 2680000)],
            [(-1, -5700000), (-1, -5700001)],
        ].into_iter().map(|list| list.iter().cloned().collect()).collect();
        let ys = vec![
            vec![(0, 2679998), (0, 2679999), (0, 2680000), (0, 2680001)],
            vec![(0, -5700000), (0, -5699999), (1, -5700000)],
            vec![(0, 5989997), (1, 5989998), (0, 5989999), (1, 5990000), (0, 5990001), (0, 5990000), (0, 5989998)],
            vec![(0, -5130000), (0, -5129999), (0, -5129998), (0, -5129997)],
            vec![(0, -4729999), (1, -4729998), (2, -4729997), (0, -4730000), (0, -4730001)],
            vec![(0, -999999), (0, -1000000), (0, -1000001), (0, -999998)],
            vec![(0, 7289999), (0, 7289998), (0, 7290000), (1, 7290001), (1, 7290002)],
            vec![(0, -1040000), (1, -1039999), (0, -1039998), (0, -1040001),(0, -1039999)],
            vec![(0, 7979999), (0, 7979998), (0, 7980000), (0, 7980001)],
            vec![(0, -2580000), (1, -2579999), (0, -2580001), (0, -2579999)],
            vec![(0, -6740000), (0, -6739999), (0, -6739998), (0, -6739997)],
            vec![(0, 4700000), (0, 4699999), (0, 4699998), (0, 4699997), (0, 4700001)],
            vec![(0, -3030000), (0, -3030001), (0, -3029999), (0, -3029998), (1, -3030000)],
            vec![(0, -3130000), (0, -3130001), (1, -3130000)],
            vec![(0, -2749998), (0, -2749999), (1, -2750000), (0, -2750001), (0, -2750000)],
            vec![(0, -6159999), (0, -6160000), (0, -6160001), (0, -6159998)],
            vec![(0, 3640000), (1, 3640001), (0, 3639999), (0, 3639998), (0, 3640001)],
        ].into_iter().map(|list| list.iter().cloned().collect()).collect();

        let new_cluster = join_clusters(xs, ys, 4);
        assert_eq!(new_cluster.len(), 16);
    }
}
