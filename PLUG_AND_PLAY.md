# PLUG_AND_PLAY — Genetic

> Evolutionary computation with ternary genomes

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
ternary-genetic = { git = "https://github.com/SuperInstance/ternary-genetic" }
```

Use in your code:

```rust
use ternary_genetic::{TernaryChromosome, evolve};

let mut pop = vec![TernaryChromosome::random(100); 50];
evolve(&mut pop, 100, |c| c.fitness());
```

## 📚 Available Documentation

| Document | Description |
|----------|-------------|
| `docs/FROM_BINARY.md` | Understanding ternary concepts as a binary programmer |
| `docs/MIGRATION.md` | Version migration guide |
| `docs/FUTURE-INTEGRATION.md` | Planned features and roadmap |

## 🔗 Integration

This crate is part of the [SuperInstance ternary fleet](https://github.com/SuperInstance). It uses the canonical `Ternary` type from `ternary-types` for cross-crate compatibility.

## 📄 License

MIT
