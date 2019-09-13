package fcc.clique;

import weka.core.Instances;
import weka.core.converters.ConverterUtils.DataSource;

import i9.subspace.base.Cluster;

import java.util.List;


public class Clique
{
    public static void main( String[] args ) throws Exception
    {
        // Assumes working directory to be 4_benchmark_java/java-clique/
        var source = new DataSource("./clique/src/resources/data_10_shuffled.arff");
        var data = source.getDataSet();

        int xi = 20;
        double tau = (double) 5 / data.size();
        List<Cluster> result = cluster(data, xi , tau);

        System.out.println(allTwoDimClusters(result).size());
    }

    public static List<Cluster> cluster(Instances data, int xi, double tau) throws Exception {
        var clique = new weka.subspaceClusterer.Clique();
        clique.setXI(xi);
        clique.setTAU(tau);

        clique.buildSubspaceClusterer(data);
        var result  = clique.getSubspaceClustering();
        return result;
    }

    public static List<Cluster> allTwoDimClusters(List<Cluster> clusters) {
        clusters.removeIf(c -> !(c.m_subspace[0] && c.m_subspace[1]));
        return clusters;
    }
}
