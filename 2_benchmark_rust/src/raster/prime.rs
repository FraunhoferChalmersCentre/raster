/*!
 * RASTER' (pronounced raster prime) is regular raster, where the input points
 * are retained with the tiles.
 */

pub mod par;

use indexmap::IndexMap;
use hashbrown::HashMap;

use crate::{
    Float,
    Point,
    Tile,
};

type TileMap = IndexMap<Tile, Vec<Point>>;


/// Prime version of [map_to_tiles](../fn.map_to_tiles.html).
pub fn map_to_tiles(
    points: &Vec<Point>,
    precision: Float,
    threshold: usize,
) -> (TileMap, Float) {
    let scalar = (10 as Float).powf(precision);
    let mut tile_map = HashMap::new();

    for p in points {
        let p_int = p.truncate(scalar);

        let ps = tile_map.entry(p_int).or_insert(vec![]);
        ps.push(*p);
    }

    let significant = tile_map
        .into_iter()
        .filter(|(_, ps)| ps.len() >= threshold )
        .collect();

    (significant, scalar)
}


/// Prime version of [cluster_tiles](../fn.cluster_tiles.html).
pub fn cluster_tiles(tiles: TileMap, min_cluster_size: usize) -> Vec<TileMap> {
    let mut to_visit = tiles;
    let mut clusters = Vec::new();

    while let Some((kx, vx)) = to_visit.pop() { // starting point for a new cluster

        let mut cluster = IndexMap::new();
        cluster.insert(kx, vx);

        let mut to_check: IndexMap<_,_> = pop_neighbors(kx, &mut to_visit).collect();
        while let Some((kc, vc)) = to_check.pop() {
            cluster.insert(kc, vc);

            let new_neighbors = pop_neighbors(kc, &mut to_visit);
            for (kn, vn) in new_neighbors {
                to_check.insert(kn, vn);
            }
        }

        if cluster.len() >= min_cluster_size {
            clusters.push(cluster);
        }
    }
    clusters
}


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

    // map neighbors to points if they exists
    candidates.into_iter().filter_map( move |n|
        if let Some(points) = tiles.remove(&n) {
            Some((n, points))
        } else {None}
    )
}
