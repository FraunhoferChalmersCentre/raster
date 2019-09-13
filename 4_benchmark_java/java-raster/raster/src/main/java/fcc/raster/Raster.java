package fcc.raster;

import java.io.FileWriter;
import java.io.IOException;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.HashMap;
import java.util.HashSet;


public class Raster {
    public static void main(String[] args) throws Exception {
        // Assumes working directory to be 4_benchmark_java/java-raster/
        var data = CSV.loadPoints("../../0_data_generators/data_100_shuffled.csv");
        double precision = 3.5;
        int threshold = 5;
        int min_size = 4;

        var projection = mapToTiles(data, precision, threshold);
        var clusters = clusteringTiles(projection, min_size);

        System.out.println(clusters.size());
    }

    public static Map<List<Integer>, Integer> mapToTiles(List<List<Double>> data
                                                        , double precision
                                                        , int threshold) {
        var scalar = Math.pow(10, precision);
        var all_points = new HashMap<List<Integer>, Integer>();

        for (List<Double> point : data) {
            var lat = (int) (point.get(0) * scalar);
            var lon = (int) (point.get(1) * scalar);
            List<Integer> tile = new ArrayList<>(2);
            tile.add(lat); tile.add(lon);

            all_points.merge(tile, 1, (v1, v2) -> v1+v2);
        }

        // Filter results to only retain tiles with at least the provided
        // threshold value of observations.
        all_points.values().removeIf(v -> v < threshold);

        return all_points;
    }

    public static final List<List<Integer>> getNeighbors(List<Integer> coordinate
                                            , Map<List<Integer>, Integer> tiles) {
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

        var result = new ArrayList<List<Integer>>();
        for (List<Integer> n : neighbors) {
            var in_tiles = tiles.remove(n);
            if (in_tiles != null) result.add(n);
        }

        return result;
    }

    public static List<Set<List<Integer>>> clusteringTiles(Map<List<Integer>, Integer> tiles, int min_size) {
        var clusters = new ArrayList<Set<List<Integer>>>();

        while (!tiles.isEmpty()) {
            // pop the first significant tile as a starting point for a new cluster
            var tile_iter = tiles.keySet().iterator();
            var x = tile_iter.next();
            tile_iter.remove();

            var visited = new HashSet<List<Integer>>();
            visited.add(x);

            var to_check = getNeighbors(x, tiles);

            while (!to_check.isEmpty()) {
                // Visit a value and fetch all neighbors
                var value = to_check.remove(0);
                visited.add(value);
                to_check.addAll(getNeighbors(value, tiles));
            }

            if (visited.size() >= min_size) {
                clusters.add(visited);
            }
        }

        return clusters;
    }

    public static void printClusters(List<Set<List<Integer>>> clusters) throws IOException {
        var csvWriter = new FileWriter("clustering.csv");
        csvWriter.append("Cluster Number");
        csvWriter.append(',');
        csvWriter.append("X-Position");
        csvWriter.append(',');
        csvWriter.append("Y-Position");
        csvWriter.append('\n');

        Integer counter = 1;
        for (Set<List<Integer>> cluster : clusters) {
            for(List<Integer> tile : cluster) {
                csvWriter.append(counter.toString());
                csvWriter.append(',');
                csvWriter.append(tile.get(0).toString());
                csvWriter.append(',');
                csvWriter.append(tile.get(1).toString());
                csvWriter.append('\n');
            }
            counter += 1;
        }
        csvWriter.flush();
        csvWriter.close();
    }
}
