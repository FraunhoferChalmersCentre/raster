package fcc.raster.prime;

import java.util.List;

public class Tile{
    public final List<Integer> tile_coordinate;
    public List<List<Double>> points;

    public Tile(List<Integer> tile_coordinate, List<List<Double>> points){
        this.tile_coordinate = tile_coordinate;
        this.points = points;
    }
}
