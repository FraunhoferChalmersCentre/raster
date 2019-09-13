package fcc.raster;

import java.io.BufferedReader;
import java.io.FileReader;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

public class CSV {

    public static List<List<Double>> loadPoints(String filepath)
            throws NumberFormatException, IOException {
        var result = new ArrayList<List<Double>>();

        var fr = new FileReader(filepath); // May throw e.g. file not found
        try(var reader = new BufferedReader(fr)){
            String row;
            while ((row = reader.readLine()) != null){
                String[] point = row.split(",");

                var x = Double.parseDouble(point[0]);
                var y = Double.parseDouble(point[1]);
                List<Double> p = new ArrayList<>(2);
                p.add(x); p.add(y);

                result.add(p);
            }
        }

        return result;
    }
}
