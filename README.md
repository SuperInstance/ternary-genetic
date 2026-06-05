# ternary-genetic

> Evolutionary computation with ternary genomes: `-1`, `0`, `+1`.

[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## What problem does this solve?

Classical genetic algorithms encode genomes as binary strings (`{0, 1}`). In computational biology, however, many regulatory processes are naturally **ternary**: gene expression can be down-regulated (`-1`), neutral (`0`), or up-regulated (`+1`); codon usage tables have three-stop-codon logic; and signaling pathways often exhibit three-level activation (inhibited / basal / activated). This crate implements a complete evolutionary framework—selection, crossover, mutation, and elitism—where the fundamental allele is a **trit** (`{-1, 0, 1}`) rather than a bit.

Mathematically, the search space is the ternary Hamming cube `{−1, 0, 1}ᴸ` with `3ᴸ` vertices. The genetic operators (single-point crossover, uniform crossover, and trit-flip mutation) define a random walk on this hypergraph, biased by fitness. This is isomorphic to optimizing over a **three-letter alphabet**, a setting that appears in codon-optimization problems, ternary neural-weight pruning, and certain constraint-satisfaction landscapes.

---

## The science

### Ternary genomes

A `TernaryChromosome` is a vector of trits:

```rust
pub type Trit = i8;  // -1, 0, or +1
```

| Trit | Biological interpretation | Machine-learning analogue |
|------|---------------------------|---------------------------|
| `-1` | Down-regulation / deletion | Negative weight / off-bit |
| ` 0` | Neutral / silent           | Zero / pruned connection  |
| `+1` | Up-regulation / insertion  | Positive weight / on-bit  |

### Fitness landscapes

The crate provides three standard landscapes via the `FitnessFunction` trait:

1. **`MaxSumFitness`** — Maximizes `Σ trits`. The global optimum is the all-`+1` genome. This is a smooth, unimodal landscape useful for sanity-checking convergence.
2. **`TargetFitness`** — Minimizes Hamming distance to a target chromosome. Fitness is `1 / (1 + d)`, so an exact match yields `1.0`. This is a needle-in-a-haystack problem with a single peak.
3. **`OneMaxFitness`** — Counts the number of `+1` trits (a ternary generalization of the classic OneMax problem).

### Genetic operators

- **Tournament selection**: Pick `k` individuals at random, return the fittest. Low selection pressure when `k` is small; high pressure when `k` approaches population size.
- **Roulette selection**: Fitness-proportional sampling with automatic shifting to handle negative fitness values.
- **Single-point crossover**: Two parents exchange suffixes at a random locus. Preserves contiguous building blocks.
- **Uniform crossover**: Each trit is inherited independently from either parent with 50 % probability. Disrupts linkage but explores combinatorially.
- **Trit-flip mutation**: Each trit is replaced by a uniformly random trit (`-1`, `0`, `+1`) with probability `rate`. This maintains exploration even when the population converges.
- **Elitism**: The top `N` individuals are cloned directly into the next generation, guaranteeing that the best fitness is monotonically non-decreasing.

### Convergence guarantees

Because the mutation operator has non-zero probability of reaching any point in the search space, the combined algorithm is an **ergodic Markov chain**: the global optimum is reachable from any state. Elitism ensures the best-so-far solution is never lost, so the expected time to absorption at the optimum is finite (though potentially exponential in `L`).

---

## Architecture

```text
┌─────────────────────────────────────────┐
│       TernaryChromosome                 │
│  genes: Vec<Trit>  |  length: usize     │
│  ├── trit_sum()                         │
│  └── hamming_distance(&other)            │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│         Individual                      │
│  chromosome + fitness: f64              │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│         Population                      │
│  individuals: Vec<Individual>           │
│  ├── evaluate(&FitnessFunction)         │
│  ├── best() / worst() / mean_fitness()  │
│  └── new(size, length, seed)            │
└─────────────┬───────────────────────────┘
              │
    ┌─────────┴─────────┐
    ▼                   ▼
┌─────────────┐   ┌──────────────┐
│ tournament  │   │   roulette   │
│_selection() │   │ _selection() │
└─────────────┘   └──────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│      GeneticAlgorithm                   │
│  population, mutation_rate, elitism     │
│  ├── step(&fitness, seed)               │
│  │     1. evaluate                      │
│  │     2. elitism                       │
│  │     3. selection → crossover → mutate│
│  └── run(generations, seed) -> &Best    │
└─────────────────────────────────────────┘
```

---

## Getting Started

Add to `Cargo.toml`:

```toml
[dependencies]
ternary-genetic = { git = "https://github.com/SuperInstance/ternary-genetic.git" }
```

Evolve a population toward an all-`+1` genome:

```rust
use ternary_genetic::{GeneticAlgorithm, MaxSumFitness};

fn main() {
    let mut ga = GeneticAlgorithm::new(
        50,      // population size
        20,      // genome length
        0.05,    // mutation rate
        2,       // elitism count
        1234,    // seed
    );

    let best = ga.run(&MaxSumFitness, 100, 5678);

    println!(
        "Generation {} | Best fitness = {:.2} | Genome = {:?}",
        ga.population.generation,
        best.fitness,
        best.chromosome.genes
    );
}
```

Compile and run:

```bash
cargo run
```

---

## Running the Tests

```bash
cargo test
```

The 14 tests exercise genotype representation, fitness evaluation, genetic operators, and evolutionary dynamics:

| Test | What it verifies |
|------|------------------|
| `chromosome_new_zeros` | A fresh chromosome is all zeros and reports the correct length. |
| `chromosome_random_range` | Random initialization produces only valid trits (`-1`, `0`, `+1`). |
| `chromosome_trit_sum` | `trit_sum()` correctly aggregates signed trits (e.g., `[1,0,-1,1] → 1`). |
| `chromosome_hamming` | `hamming_distance()` counts position-wise mismatches between two chromosomes. |
| `max_sum_fitness` | `MaxSumFitness` maps `all(+1)` to `+L` and `all(-1)` to `−L`. |
| `target_fitness_exact` | `TargetFitness` returns exactly `1.0` when the chromosome matches the target perfectly. |
| `target_fitness_partial` | A Hamming distance of 1 yields fitness `0.5` (`1/(1+1)`). |
| `onemax_fitness` | `OneMaxFitness` counts only `+1` alleles (e.g., `[1,0,-1,1] → 2.0`). |
| `single_point_crossover_basic` | Single-point crossover produces correct prefix/suffix recombination. |
| `uniform_crossover_lengths` | Uniform crossover preserves chromosome length (`20` trits in, `20` out). |
| `trit_flip_rate_zero` | Zero mutation rate leaves the chromosome completely unchanged. |
| `population_best` | After evaluation, `best()` returns the individual with maximal fitness. |
| `ga_improves_fitness` | Over 20 generations, mean population fitness under `MaxSumFitness` does not decrease. |
| `elitism_preserves_best` | After one `step()`, an individual with fitness ≥ the pre-step best still exists in the population. |

---

## Related crates in the ternary ecosystem

- [`ternary-ga`](https://github.com/SuperInstance/ternary-ga) — Alternative genetic algorithm implementations with multi-objective and island-model extensions.
- [`ternary-genome`](https://github.com/SuperInstance/ternary-genome) — Ternary genome data structures, compression, and serialization utilities.
- [`ternary-evolution-advanced`](https://github.com/SuperInstance/ternary-evolution-advanced) — Co-evolution, speciation, and niching operators for ternary populations.
- [`ternary-fitness`](https://github.com/SuperInstance/ternary-fitness) — Landscape analysis: ruggedness, epistasis, and neutrality measures for ternary fitness functions.

---

## License

This project is licensed under the [MIT License](LICENSE).
