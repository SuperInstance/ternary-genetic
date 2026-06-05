# From Binary to Ternary: Genetic Algorithms

## The Trap

Classical genetic algorithms encode genomes as binary strings: `{0, 1}`. Each gene is a bit — on or off, present or absent. This works for problems with binary decisions, but biology itself is not binary. Gene expression can be down-regulated, up-regulated, or *silent*. A transcription factor doesn't just bind or not bind — it can activate, repress, or have no effect. By encoding everything as a bit, you're forcing ternary biology into a binary straightjacket.

The practical consequence: binary genomes can't model neutral mutations — changes that don't affect fitness but open new evolutionary paths. In a binary GA, every mutation flips a bit. There's no "this gene doesn't matter right now" state. The search space is either `{0, 1}ⁿ` when the real space is `{-1, 0, +1}ⁿ`.

## Map to Three States

| Domain | −1 | 0 | +1 |
|--------|----|---|-----|
| Gene expression | down-regulated | silent / neutral | up-regulated |
| Neural weight | negative | pruned / zero | positive |
| Decision | avoid | abstain | select |
| Mutation effect | flip to −1 | flip to 0 | flip to +1 |

## From Binary to Ternary

**Before: binary genome**

```rust
#[derive(Clone)]
struct BinaryChromosome {
    genes: Vec<bool>,  // each gene is on or off
}
// A neutral mutation is impossible
// Every change has immediate phenotypic effect
```

**After: ternary genome**

```rust
type Trit = i8;  // -1, 0, or +1

struct TernaryChromosome {
    genes: Vec<Trit>,  // down, silent, up
}
// The space is {-1, 0, +1}ⁿ — 3ⁿ points
// Instead of 2ⁿ
```

The neutral state (`0`) is the evolutionary secret weapon. In a binary GA, every mutation changes the phenotype. In a ternary GA, a mutation that flips a gene between `+1` and `-1` (passing through `0`) creates a *bridge* — the gene can be silent in intermediate generations while the rest of the genome adapts.

**Before: trit-flip only changes between 0 and 1**

**After: trit-flip traverses all three states**

```rust
fn trit_flip_mutate(chromosome: &mut TernaryChromosome, rate: f64) {
    for trit in &mut chromosome.genes {
        if random() < rate {
            // Replace with uniformly random trit: -1, 0, or +1
            *trit = random_trit();
        }
    }
}
```

Three outcomes per mutation, not two. The probability landscape is richer.

**Selection before vs. after:**

Binary tournament selection picks the better of two individuals. But with binary fitness, two individuals with the same fitness are indistinguishable — even if one has more neutral `0` genes that could later flip to useful values. Ternary tournament selection can prefer individuals with more neutral genes when fitness is tied, since they represent greater evolutionary potential.

**0 is not nothing:** In a binary genome, a gene that's off is simply absent. In a ternary genome, a `0` gene is *available* — it can become active or repressive in one mutation. The neutral state is an evolutionary reservoir, preserving diversity without hurting current fitness. This is exactly how biological neutral networks work.

## Why It Matters

Ternary genomes model biology more faithfully and create richer fitness landscapes. The `0` state enables neutral exploration — the genome can traverse fitness-neutral paths to reach new fitness peaks, avoiding local optima that trap binary GAs. The search space is larger (3ⁿ vs 2ⁿ), but the structure is more aligned with real evolutionary dynamics.
