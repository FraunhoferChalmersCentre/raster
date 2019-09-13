package fcc.raster.prime;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;


public class RasterPrime {
    public static void main(String[] args) throws Exception {
        // Assumes working directory to be 4_benchmark_java/java-raster-prime/
        var data = CSV.loadPoints("../../0_data_generators/data_100_shuffled.csv");
        double precision = 3.5;
        int threshold = 5;
        int min_size = 4;

        var projection = mapToTiles(data, precision, threshold);
        var clusters = clusteringTiles(projection, min_size);

        System.out.println(clusters.size());
    }

    public static Map<List<Integer>, List<List<Double>>> mapToTiles(List<List<Double>> data
                                                        , double precision
                                                        , int threshold) {
        var scalar = Math.pow(10, precision);
        var all_points = new HashMap<List<Integer>, List<List<Double>>>();

        for (List<Double> point : data) {
            var lat = (int) (point.get(0) * scalar);
            var lon = (int) (point.get(1) * scalar);
            List<Integer> tile = new ArrayList<>(2);
            tile.add(lat); tile.add(lon);

            var lookup = all_points.get(tile);
            if (lookup == null) {
                lookup = new ArrayList<List<Double>>();
            }
            lookup.add(point);
            all_points.put(tile, lookup);
        }

        // Only retain tiles with at least the provided
        // threshold value of observations.
        all_points.values().removeIf(v -> v.size() < threshold);

        return all_points;
    }

    public static final List<Tile> getNeighbors(List<Integer> coordinate
                                            , Map<List<Integer>, List<List<Double>>> tiles) {
        var x = coordinate.get(0);
        var y = coordinate.get(1);

        var neighbors = new ArrayList<List<Integer>>(8);
        neighbors.add(Arrays.asList(x+1, y  ));
        neighbors.add(Arrays.asList(x-1, y  ));
        neighbors.add(Arrays.asList(x  , y+1));
        neighbors.add(Arrays.asList(x  , y-1));
        neighbors.add(Arrays.asList(x+1, y-1));
        neighbors.add(Arrays.asList(x+1, y+1));
        neighbors.add(Arrays.asList(x-1, y-1));
        neighbors.add(Arrays.asList(x-1, y+1));

        var result = new ArrayList<Tile>();
        for (List<Integer> n : neighbors) {
            var in_tiles = tiles.remove(n);
            if (in_tiles != null) result.add(new Tile(n, in_tiles));
        }
        return result;
    }

    public static List<Map<List<Integer>, List<List<Double>>>> clusteringTiles(Map<List<Integer>, List<List<Double>>> projection,
            int min_size) {
        var clusters = new ArrayList< Map<List<Integer>, List<List<Double>>>>();

        while (!projection.isEmpty()) {
            // pop the first significant tile as a starting point for a new cluster
            var tile_iter = projection.entrySet().iterator();
            var x = tile_iter.next();
            tile_iter.remove();

            var visited = new HashMap<List<Integer>, List<List<Double>>>();
            visited.put(x.getKey(), x.getValue());

            var to_check = getNeighbors(x.getKey(), projection);

            while (!to_check.isEmpty()) {
                // Visit a value and fetch all neighbors
                var value = to_check.remove(0);
                visited.put(value.tile_coordinate, value.points);
                to_check.addAll(getNeighbors(value.tile_coordinate, projection));
            }

            if (visited.size() >= min_size) {
                clusters.add(visited);
            }
        }

        return clusters;
    }
}
