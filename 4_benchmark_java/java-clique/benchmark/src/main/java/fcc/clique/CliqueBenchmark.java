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

package fcc.clique;

import weka.core.Instances;
import weka.core.converters.ConverterUtils.DataSource;
import i9.subspace.base.Cluster;

import org.openjdk.jmh.annotations.*;

import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;
import java.util.zip.DataFormatException;

import fcc.clique.Clique;


@BenchmarkMode(Mode.AverageTime)
@OutputTimeUnit(TimeUnit.SECONDS)
@Measurement(time=1, timeUnit=TimeUnit.MILLISECONDS)
@State(Scope.Benchmark)
public class CliqueBenchmark {

    Instances data;
    Map<Integer, Integer> params = Map.of(10, 20, 100, 300, 1000, 1000); // map from #clusters -> xi-value
    int xi;
    List<Cluster> result;

    @Param({"10", "100", "1000"})
    public int n_clusters;

    @Setup(Level.Trial)
    public void loadData() throws Exception {
        // Assumes working directory to be 4_benchmark_java/java-clique/
        var source = new DataSource("./clique/src/resources/data_" + String.valueOf(n_clusters) + "_shuffled.arff");
        data = source.getDataSet();

        // Sanity-check
        if (data.size()/500 != n_clusters){
            throw new DataFormatException("Got " + data.size() + " data points, but expected " + (n_clusters*500));
        }

        xi = params.get(n_clusters);
    }

    @TearDown(Level.Trial)
    public void printNumClusters() {
        var n_clusters = Clique.allTwoDimClusters(result);
        System.out.println("\nFound " + n_clusters.size() + " clusters");
    }

    @Benchmark
    public void bm() throws Exception {
        double tau = (double) 5 / data.size();
        result = Clique.cluster(data, xi , tau);
    }
}
