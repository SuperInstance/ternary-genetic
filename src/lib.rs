#![allow(dead_code)]

/// Canonical ternary type — re-exported from [ternary-types](https://github.com/SuperInstance/ternary-types).
pub use ternary_types::Ternary;

/// Deprecated: use [`Ternary`] instead.
#[deprecated(since = "0.2.0", note = "use ternary_types::Ternary instead")]
pub type Trit = i8;

// ---------------------------------------------------------------------------
// LCG random number generator (no external deps)
// ---------------------------------------------------------------------------

fn lcg_next(state: &mut u64) -> u64 {
    // Knuth's constants
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *state
}

/// Return a random trit in {-1, 0, 1} advancing `state`.
fn random_trit(state: &mut u64) -> Trit {
    let v = lcg_next(state);
    match v % 3 {
        0 => -1,
        1 => 0,
        _ => 1,
    }
}

/// Return a random usize in [0, upper) advancing `state`.
fn random_usize(state: &mut u64, upper: usize) -> usize {
    (lcg_next(state) as usize) % upper
}

/// Return a random f64 in [0.0, 1.0) advancing `state`.
fn random_f64(state: &mut u64) -> f64 {
    (lcg_next(state) as f64) / (u64::MAX as f64)
}

// ---------------------------------------------------------------------------
// TernaryChromosome
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct TernaryChromosome {
    pub genes: Vec<Trit>,
    pub length: usize,
}

impl TernaryChromosome {
    /// Create a chromosome of `length` trits, all set to 0.
    pub fn new(length: usize) -> Self {
        Self {
            genes: vec![0; length],
            length,
        }
    }

    /// Create a chromosome with random trits using a simple LCG seeded by `seed`.
    pub fn random(length: usize, seed: u64) -> Self {
        let mut state = seed;
        let genes: Vec<Trit> = (0..length).map(|_| random_trit(&mut state)).collect();
        Self { genes, length }
    }

    /// Create a chromosome from an existing vector.
    pub fn from_vec(genes: Vec<Trit>) -> Self {
        let length = genes.len();
        Self { genes, length }
    }

    /// Get trit at position `i`.
    pub fn get(&self, i: usize) -> Trit {
        self.genes[i]
    }

    /// Set trit at position `i` to `val`.
    pub fn set(&mut self, i: usize, val: Trit) {
        self.genes[i] = val;
    }

    /// Sum of all trits.
    pub fn trit_sum(&self) -> i64 {
        self.genes.iter().map(|&t| t as i64).sum()
    }

    /// Number of positions where `self` and `other` differ.
    pub fn hamming_distance(&self, other: &TernaryChromosome) -> usize {
        assert_eq!(self.length, other.length, "chromosomes must have equal length");
        self.genes
            .iter()
            .zip(other.genes.iter())
            .filter(|(a, b)| a != b)
            .count()
    }
}

// ---------------------------------------------------------------------------
// Fitness functions
// ---------------------------------------------------------------------------

pub trait FitnessFunction {
    fn evaluate(&self, chromosome: &TernaryChromosome) -> f64;
}

/// Maximizes trit sum (optimal = all +1).
pub struct MaxSumFitness;

impl FitnessFunction for MaxSumFitness {
    fn evaluate(&self, chromosome: &TernaryChromosome) -> f64 {
        chromosome.trit_sum() as f64
    }
}

/// Minimizes hamming distance to a target chromosome.
pub struct TargetFitness {
    pub target: TernaryChromosome,
}

impl FitnessFunction for TargetFitness {
    fn evaluate(&self, chromosome: &TernaryChromosome) -> f64 {
        let d = chromosome.hamming_distance(&self.target) as f64;
        1.0 / (1.0 + d)
    }
}

/// OneMax variant: counts the number of +1 trits.
pub struct OneMaxFitness;

impl FitnessFunction for OneMaxFitness {
    fn evaluate(&self, chromosome: &TernaryChromosome) -> f64 {
        chromosome.genes.iter().filter(|&&t| t == 1).count() as f64
    }
}

// ---------------------------------------------------------------------------
// Individual
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct Individual {
    pub chromosome: TernaryChromosome,
    pub fitness: f64,
}

impl Individual {
    pub fn new(chromosome: TernaryChromosome, fitness: f64) -> Self {
        Self { chromosome, fitness }
    }
}

// ---------------------------------------------------------------------------
// Population
// ---------------------------------------------------------------------------

pub struct Population {
    pub individuals: Vec<Individual>,
    pub generation: u64,
}

impl Population {
    /// Create a random population of `size` individuals each with `length` trits.
    pub fn new(size: usize, length: usize, seed: u64) -> Self {
        let mut state = seed;
        let individuals = (0..size)
            .map(|i| {
                let chr_seed = lcg_next(&mut state) ^ (i as u64).wrapping_mul(0x9e3779b97f4a7c15);
                let chromosome = TernaryChromosome::random(length, chr_seed);
                Individual::new(chromosome, 0.0)
            })
            .collect();
        Self { individuals, generation: 0 }
    }

    /// Return the individual with the highest fitness.
    pub fn best(&self) -> &Individual {
        self.individuals
            .iter()
            .max_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
            .expect("population is empty")
    }

    /// Return the individual with the lowest fitness.
    pub fn worst(&self) -> &Individual {
        self.individuals
            .iter()
            .min_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap())
            .expect("population is empty")
    }

    /// Mean fitness across all individuals.
    pub fn mean_fitness(&self) -> f64 {
        if self.individuals.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.individuals.iter().map(|ind| ind.fitness).sum();
        sum / self.individuals.len() as f64
    }

    /// Evaluate every individual using `fitness_fn`, updating their fitness field.
    pub fn evaluate<F: FitnessFunction>(&mut self, fitness_fn: &F) {
        for ind in self.individuals.iter_mut() {
            ind.fitness = fitness_fn.evaluate(&ind.chromosome);
        }
    }

    /// Number of individuals in the population.
    pub fn size(&self) -> usize {
        self.individuals.len()
    }
}

// ---------------------------------------------------------------------------
// Selection
// ---------------------------------------------------------------------------

/// Tournament selection: pick `tournament_size` random individuals, return the fittest.
pub fn tournament_selection<'a>(
    population: &'a Population,
    tournament_size: usize,
    seed: u64,
) -> &'a Individual {
    assert!(!population.individuals.is_empty(), "population is empty");
    let size = tournament_size.max(1).min(population.size());
    let mut state = seed;
    let mut best_idx = random_usize(&mut state, population.size());
    for _ in 1..size {
        let idx = random_usize(&mut state, population.size());
        if population.individuals[idx].fitness > population.individuals[best_idx].fitness {
            best_idx = idx;
        }
    }
    &population.individuals[best_idx]
}

/// Roulette (fitness-proportional) selection. Falls back to uniform if total fitness <= 0.
pub fn roulette_selection<'a>(population: &'a Population, seed: u64) -> &'a Individual {
    assert!(!population.individuals.is_empty(), "population is empty");
    let mut state = seed;

    // Shift fitnesses so the minimum is 0 (handles negative fitness).
    let min_f = population
        .individuals
        .iter()
        .map(|ind| ind.fitness)
        .fold(f64::INFINITY, f64::min);
    let shifted: Vec<f64> = population
        .individuals
        .iter()
        .map(|ind| ind.fitness - min_f)
        .collect();
    let total: f64 = shifted.iter().sum();

    if total <= 0.0 {
        // Uniform fallback
        let idx = random_usize(&mut state, population.size());
        return &population.individuals[idx];
    }

    let pick = random_f64(&mut state) * total;
    let mut cumulative = 0.0;
    for (i, &s) in shifted.iter().enumerate() {
        cumulative += s;
        if pick <= cumulative {
            return &population.individuals[i];
        }
    }
    // Fallback to last individual (floating-point rounding edge case)
    population.individuals.last().unwrap()
}

// ---------------------------------------------------------------------------
// Crossover
// ---------------------------------------------------------------------------

/// Single-point crossover at `point`: children swap tails after position `point`.
pub fn single_point_crossover(
    parent_a: &TernaryChromosome,
    parent_b: &TernaryChromosome,
    point: usize,
) -> (TernaryChromosome, TernaryChromosome) {
    assert_eq!(parent_a.length, parent_b.length, "parents must have equal length");
    let len = parent_a.length;
    let point = point.min(len);

    let mut child_a = vec![0i8; len];
    let mut child_b = vec![0i8; len];

    child_a[..point].copy_from_slice(&parent_a.genes[..point]);
    child_a[point..].copy_from_slice(&parent_b.genes[point..]);

    child_b[..point].copy_from_slice(&parent_b.genes[..point]);
    child_b[point..].copy_from_slice(&parent_a.genes[point..]);

    (
        TernaryChromosome::from_vec(child_a),
        TernaryChromosome::from_vec(child_b),
    )
}

/// Uniform crossover: each trit independently chosen from parent_a or parent_b with 50/50 chance.
pub fn uniform_crossover(
    parent_a: &TernaryChromosome,
    parent_b: &TernaryChromosome,
    seed: u64,
) -> (TernaryChromosome, TernaryChromosome) {
    assert_eq!(parent_a.length, parent_b.length, "parents must have equal length");
    let len = parent_a.length;
    let mut state = seed;

    let mut child_a = vec![0i8; len];
    let mut child_b = vec![0i8; len];

    for i in 0..len {
        let v = lcg_next(&mut state);
        if v % 2 == 0 {
            child_a[i] = parent_a.genes[i];
            child_b[i] = parent_b.genes[i];
        } else {
            child_a[i] = parent_b.genes[i];
            child_b[i] = parent_a.genes[i];
        }
    }

    (
        TernaryChromosome::from_vec(child_a),
        TernaryChromosome::from_vec(child_b),
    )
}

// ---------------------------------------------------------------------------
// Mutation
// ---------------------------------------------------------------------------

/// Trit-flip mutation: each trit is replaced with a random trit with probability `rate`.
pub fn trit_flip_mutation(
    chromosome: &TernaryChromosome,
    rate: f64,
    seed: u64,
) -> TernaryChromosome {
    let mut state = seed;
    let genes: Vec<Trit> = chromosome
        .genes
        .iter()
        .map(|&t| {
            let p = random_f64(&mut state);
            if p < rate {
                random_trit(&mut state)
            } else {
                t
            }
        })
        .collect();
    TernaryChromosome::from_vec(genes)
}

// ---------------------------------------------------------------------------
// Genetic Algorithm
// ---------------------------------------------------------------------------

pub struct GeneticAlgorithm {
    pub population: Population,
    pub mutation_rate: f64,
    pub elitism_count: usize,
}

impl GeneticAlgorithm {
    /// Create a new GA with a random initial population.
    pub fn new(
        pop_size: usize,
        genome_length: usize,
        mutation_rate: f64,
        elitism_count: usize,
        seed: u64,
    ) -> Self {
        Self {
            population: Population::new(pop_size, genome_length, seed),
            mutation_rate,
            elitism_count,
        }
    }

    /// Advance one generation.
    pub fn step<F: FitnessFunction>(&mut self, fitness_fn: &F, seed: u64) {
        // 1. Evaluate fitness of current population
        self.population.evaluate(fitness_fn);

        let pop_size = self.population.size();
        let genome_length = self.population.individuals[0].chromosome.length;
        let mut state = seed;

        // Sort by descending fitness for elitism
        let mut sorted: Vec<Individual> = self.population.individuals.clone();
        sorted.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());

        let mut next_gen: Vec<Individual> = Vec::with_capacity(pop_size);

        // 2. Elitism: carry forward the top `elitism_count` individuals
        let elite = self.elitism_count.min(pop_size);
        for i in 0..elite {
            next_gen.push(sorted[i].clone());
        }

        // 3. Fill the rest via tournament selection + crossover + mutation
        let tournament_size = 3.min(pop_size);
        while next_gen.len() < pop_size {
            let s_a = lcg_next(&mut state);
            let s_b = lcg_next(&mut state);
            let parent_a = tournament_selection(&self.population, tournament_size, s_a).clone();
            let parent_b = tournament_selection(&self.population, tournament_size, s_b).clone();

            let crossover_point = (lcg_next(&mut state) as usize) % (genome_length + 1);
            let (child_a, child_b) = single_point_crossover(
                &parent_a.chromosome,
                &parent_b.chromosome,
                crossover_point,
            );

            let s_ma = lcg_next(&mut state);
            let s_mb = lcg_next(&mut state);
            let mutated_a = trit_flip_mutation(&child_a, self.mutation_rate, s_ma);
            let mutated_b = trit_flip_mutation(&child_b, self.mutation_rate, s_mb);

            next_gen.push(Individual::new(mutated_a, 0.0));
            if next_gen.len() < pop_size {
                next_gen.push(Individual::new(mutated_b, 0.0));
            }
        }

        // 4. Increment generation counter
        self.population.individuals = next_gen;
        self.population.generation += 1;
    }

    /// Run `generations` steps and return the best individual found.
    pub fn run<F: FitnessFunction>(
        &mut self,
        fitness_fn: &F,
        generations: u64,
        seed: u64,
    ) -> &Individual {
        let mut state = seed;
        for _ in 0..generations {
            let step_seed = lcg_next(&mut state);
            self.step(fitness_fn, step_seed);
        }
        // Final evaluation so the returned best reflects actual fitness
        self.population.evaluate(fitness_fn);
        self.population.best()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // 1. new chromosome is all zeros
    #[test]
    fn chromosome_new_zeros() {
        let c = TernaryChromosome::new(8);
        assert_eq!(c.length, 8);
        assert!(c.genes.iter().all(|&t| t == 0));
    }

    // 2. random chromosome has all trits in {-1, 0, 1}
    #[test]
    fn chromosome_random_range() {
        let c = TernaryChromosome::random(100, 42);
        assert!(c.genes.iter().all(|&t| t == -1 || t == 0 || t == 1));
    }

    // 3. trit_sum of [1,0,-1,1] = 1
    #[test]
    fn chromosome_trit_sum() {
        let c = TernaryChromosome::from_vec(vec![1, 0, -1, 1]);
        assert_eq!(c.trit_sum(), 1);
    }

    // 4. hamming_distance([1,0,-1], [1,1,-1]) = 1
    #[test]
    fn chromosome_hamming() {
        let a = TernaryChromosome::from_vec(vec![1, 0, -1]);
        let b = TernaryChromosome::from_vec(vec![1, 1, -1]);
        assert_eq!(a.hamming_distance(&b), 1);
    }

    // 5. MaxSumFitness: [1,1,1] -> 3.0, [-1,-1,-1] -> -3.0
    #[test]
    fn max_sum_fitness() {
        let f = MaxSumFitness;
        let all_pos = TernaryChromosome::from_vec(vec![1, 1, 1]);
        let all_neg = TernaryChromosome::from_vec(vec![-1, -1, -1]);
        assert_eq!(f.evaluate(&all_pos), 3.0);
        assert_eq!(f.evaluate(&all_neg), -3.0);
    }

    // 6. TargetFitness exact match -> 1.0
    #[test]
    fn target_fitness_exact() {
        let target = TernaryChromosome::from_vec(vec![1, 0, -1, 1]);
        let chr = target.clone();
        let f = TargetFitness { target };
        assert!((f.evaluate(&chr) - 1.0).abs() < 1e-9);
    }

    // 7. TargetFitness 1 away -> 0.5
    #[test]
    fn target_fitness_partial() {
        let target = TernaryChromosome::from_vec(vec![1, 0, -1]);
        let chr = TernaryChromosome::from_vec(vec![1, 1, -1]); // 1 position differs
        let f = TargetFitness { target };
        assert!((f.evaluate(&chr) - 0.5).abs() < 1e-9);
    }

    // 8. OneMaxFitness: [1,0,-1,1] -> 2.0
    #[test]
    fn onemax_fitness() {
        let f = OneMaxFitness;
        let c = TernaryChromosome::from_vec(vec![1, 0, -1, 1]);
        assert_eq!(f.evaluate(&c), 2.0);
    }

    // 9. single_point_crossover: children are correct prefix/suffix splits
    #[test]
    fn single_point_crossover_basic() {
        let a = TernaryChromosome::from_vec(vec![1, 1, 1, 1]);
        let b = TernaryChromosome::from_vec(vec![-1, -1, -1, -1]);
        let (ca, cb) = single_point_crossover(&a, &b, 2);
        assert_eq!(ca.genes, vec![1, 1, -1, -1]);
        assert_eq!(cb.genes, vec![-1, -1, 1, 1]);
    }

    // 10. uniform_crossover children have same length as parents
    #[test]
    fn uniform_crossover_lengths() {
        let a = TernaryChromosome::random(20, 1);
        let b = TernaryChromosome::random(20, 2);
        let (ca, cb) = uniform_crossover(&a, &b, 3);
        assert_eq!(ca.length, 20);
        assert_eq!(cb.length, 20);
    }

    // 11. trit_flip_mutation with rate 0.0 returns identical chromosome
    #[test]
    fn trit_flip_rate_zero() {
        let c = TernaryChromosome::random(30, 99);
        let mutated = trit_flip_mutation(&c, 0.0, 7);
        assert_eq!(c.genes, mutated.genes);
    }

    // 12. population.best() returns individual with highest fitness
    #[test]
    fn population_best() {
        let mut pop = Population::new(10, 5, 17);
        let f = MaxSumFitness;
        pop.evaluate(&f);
        let best = pop.best();
        let max_fitness = pop
            .individuals
            .iter()
            .map(|ind| ind.fitness)
            .fold(f64::NEG_INFINITY, f64::max);
        assert_eq!(best.fitness, max_fitness);
    }

    // 13. GA improves mean fitness over 20 generations with MaxSumFitness
    #[test]
    fn ga_improves_fitness() {
        let mut ga = GeneticAlgorithm::new(30, 10, 0.05, 2, 1234);
        let f = MaxSumFitness;
        ga.population.evaluate(&f);
        let initial_mean = ga.population.mean_fitness();
        ga.run(&f, 20, 5678);
        let final_mean = ga.population.mean_fitness();
        assert!(
            final_mean >= initial_mean,
            "expected improvement: initial={:.3} final={:.3}",
            initial_mean,
            final_mean
        );
    }

    // 14. Elitism preserves the best individual after one step
    #[test]
    fn elitism_preserves_best() {
        let mut ga = GeneticAlgorithm::new(20, 8, 0.1, 2, 42);
        let f = MaxSumFitness;
        ga.population.evaluate(&f);
        let best_before = ga.population.best().fitness;
        ga.step(&f, 999);
        ga.population.evaluate(&f);
        let exists = ga
            .population
            .individuals
            .iter()
            .any(|ind| ind.fitness >= best_before);
        assert!(
            exists,
            "best individual from previous gen (fitness={:.3}) not found after step",
            best_before
        );
    }
}
