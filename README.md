# ternary-genetic

**Genetic algorithms with ternary {-1, 0, +1} chromosomes: neutral genes, trit-flip mutation, richer search spaces.**

Standard genetic algorithms use binary chromosomes where each gene is 0 or 1. This crate uses ternary chromosomes where each gene is -1, 0, or +1. The "0" gene is **neutral** — it doesn't contribute to or detract from fitness, creating a smoother fitness landscape where mutations can explore without immediately being penalized.

---

## Why Ternary Chromosomes?

**Search space**: A binary chromosome of length N has 2^N configurations. A ternary chromosome has 3^N — that's 1.585× more configurations per gene position.

For N=100: binary = 2^100 ≈ 1.27×10^30, ternary = 3^100 ≈ 5.15×10^47.

The neutral gene (0) creates **neutral networks** in the fitness landscape — connected regions of equal fitness where the population can drift without selection pressure. This is known to help avoid local optima (Huynen et al., 1996).

**Mutation**: Binary bit-flip has 1 possible outcome (0↔1). Ternary trit-flip has 2 possible outcomes (-1→0, -1→+1, etc.), maintaining more genetic diversity per mutation event.

---

## Architecture

- **`TernaryChromosome`** — Vector of {-1, 0, +1} genes with random generation
- **`FitnessFunction`** trait — Implement for your optimization problem
- **`Population`** / **`Individual`** — Population management
- **Selection**: `tournament_selection()`, `roulette_selection()`
- **Crossover**: `single_point_crossover()`, `uniform_crossover()`
- **Mutation**: `trit_flip_mutation()` — flip one trit to a different value
- **`GeneticAlgorithm`** — Full GA loop: init → evaluate → select → crossover → mutate → repeat

---

## Quick Start

```rust
use ternary_genetic::{GeneticAlgorithm, TernaryChromosome, OneMaxFitness};

let mut ga = GeneticAlgorithm::new(
    100,    // population size
    50,     // chromosome length
    0.8,    // crossover rate
    0.05,   // mutation rate
    42,     // seed
);

let best = ga.run(OneMaxFitness, 200);
println!("Best fitness: {}", best.fitness);
println!("Best chromosome: {:?}", best.chromosome.genes);
```

---

## Ecosystem

- **ternary-evolve** — Evolutionary strategies
- **ternary-step** — Step functions for iterative optimization
- **ternary-tournament** — Tournament selection variants
- **ternary-optimize** — Optimization benchmark suite

## License

MIT
