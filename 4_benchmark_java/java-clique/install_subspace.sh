#! /bin/sh

mvn install:install-file \
    -Dfile=i9-subspace.jar \
    -DgroupId=fcc.weka.subspace \
    -DartifactId=subspace \
    -DgeneratePom=true \
    -Dpackaging=jar \
    -Dversion=1.0

mvn install:install-file \
    -Dfile=i9-weka.jar \
    -DgroupId=fcc.weka.subspace \
    -DartifactId=weka \
    -DgeneratePom=true \
    -Dpackaging=jar \
    -Dversion=1.0
