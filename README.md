# Differential Datalog Benchmarks

This repository contains some simple benchmarks based on artificially generated datasets for
the [Differential Datalog engine](https://github.com/vmware/differential-datalog).

## Prerequisites

* Rust >= 1.50
* Python >= 3.8

## How-To

1) Follow Differential
   Datalog's [installation instructions](https://github.com/vmware/differential-datalog#installing-ddlog-from-a-binary-release)

2) Have a look at [query.dl](query.dl) and choose which query you would like to run.

3) Use the provided [Makefile](Makefile) to compile the query, generate the datasets, and run the query against on of
   the datasets. Each dataset also has a trace of updates associated with it (see [updates/](updates)), which will be
   fed stepwise to the query computations, and thereby showcase the incremental computation aspects.

4) You can inspect the output of the query in the respective `.out` files that will be generated. The benchmark results
   are visible in the console.

As a shorthand, you can use `make all` to execute the default query (computing the roots) over all datasets after you
have installed Differential Datalog.
