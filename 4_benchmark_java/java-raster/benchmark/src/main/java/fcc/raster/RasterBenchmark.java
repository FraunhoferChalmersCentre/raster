/*
 * Copyright (c) 2014, Oracle America, Inc.
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 *  * Redistributions of source code must retain the above copyright notice,
 *    this list of conditions and the following disclaimer.
 *
 *  * Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 *  * Neither the name of Oracle nor the names of its contributors may be used
 *    to endorse or promote products derived from this software without
 *    specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
 * AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
 * IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
 * LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
 * CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
 * SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
 * INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
 * CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF
 * THE POSSIBILITY OF SUCH DAMAGE.
 */
package fcc.raster;

import org.openjdk.jmh.annotations.*;

import java.util.List;
import java.util.Map;
import java.util.Set;
import java.io.IOException;
import java.util.concurrent.TimeUnit;
import java.util.zip.DataFormatException;

import fcc.raster.Raster;
import fcc.raster.CSV;

/**
 * Cf. https://hg.openjdk.java.net/code-tools/jmh/file/tip/jmh-samples/src/main/java/org/openjdk/jmh/samples/
 * for many useful samples of how to correctly setup JMH benchmarks.
 */

@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.MILLISECONDS)
@Measurement(time=1, timeUnit=TimeUnit.MILLISECONDS)
public class RasterBenchmark {

    @State(Scope.Benchmark)
    public static class DataState {
        public List<List<Double>> data;

        @Param({"10", "100", "1000"})
        int n_clusters;

        @Param({"3.5", "4"})
        double precision;

        int min_size  = 4;
        int threshold = 5;

        @Setup(Level.Trial)
        public void setup() throws IOException, DataFormatException {
            data = CSV.loadPoints("../../0_data_generators/data_" + String.valueOf(n_clusters) + "_shuffled.csv");

            // Sanity-check
            if (data.size()/500 != n_clusters){
                throw new DataFormatException("Got " + data.size() + " data points, but expected " + (n_clusters*500));
            }
        }
    }

    @State(Scope.Benchmark)
    public static class ClusterState extends DataState {
        public  List<Set<List<Integer>>> clustering;

        @TearDown(Level.Trial)
        public void print_n_clusters() {
            System.out.println("\nFound " + clustering.size() + " clusters");
        }
    }

    @State(Scope.Benchmark)
    public static class PreProjectState extends ClusterState {
        public Map<List<Integer>, Integer> projection;

        @Setup(Level.Invocation)
        public void setupProjection() throws IOException {
            projection = Raster.mapToTiles(data, precision, threshold);
        }
    }

    /**
     * Only benchmark the projection-step
     */
/*     @Benchmark
    public Map<List<Integer>, Integer> bm_projection(DataState state) {
        var projection = Raster.mapToTiles(state.data, state.precision, state.threshold);
        return projection;
    } */

    /**
     * Only benchmark the agglomeration-step
     */
/*     @Benchmark
    public void bm_clustering(PreProjectState proj) {
        proj.clustering = Raster.clusteringTiles(proj.projection, proj.min_size);
    } */

    /**
     * Benchmark the whole RASTER-algorithm
     */
    @Benchmark
    public void bm_raster(ClusterState state) {
        var projection = Raster.mapToTiles(state.data, state.precision, state.threshold);
        state.clustering = Raster.clusteringTiles(projection, state.min_size);
    }
}
