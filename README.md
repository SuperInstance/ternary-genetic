ternary-genetic
===============

A Rust library implementing genetic algorithms over ternary genomes, where each gene (trit) takes one of three values: -1, 0, or +1. The crate provides `TernaryChromosome` for encoding ternary individuals, three ready-made fitness functions (`MaxSumFitness`, `TargetFitness`, `OneMaxFitness`), tournament and roulette-wheel selection, single-point and uniform crossover, trit-flip mutation, and a `GeneticAlgorithm` runner with configurable elitism — all implemented from scratch with no external dependencies using a fast LCG pseudo-random number generator.
