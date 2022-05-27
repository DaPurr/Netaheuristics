//! ## Description
//! This crate's intended purpose is to offer a modular framework for popular metaheuristics, while simultaneously offering default modules such that the user only needs to focus on the problem-specific aspects of heuristics design.
//!
//! Custom modules can be built in order to fit the user's needs.
//!
//! ## Plan
//! At the least the following heuristics are planned to be implemented:
//! - Variable Neighborhood Search
//! - Simulated Annealing
//! - Large Neighborhood Search
//!
//! and their adaptive variants.
use std::{
    cell::RefCell,
    ops::SubAssign,
    time::{Duration, SystemTime},
};

use rand::Rng;

pub mod lns;
pub mod sa;
pub mod termination;
#[cfg(test)]
mod test;
pub mod vns;

/// Evaluate the quality of a solution.
pub trait Evaluate {
    fn evaluate(&self) -> f32;
}

/// A local search operator returns the neighborhood of its argument.
pub trait Operator {
    type Solution: Evaluate;
    /// Construct the neighborhood of ```solution```.
    fn construct_neighborhood(
        &self,
        solution: Self::Solution,
    ) -> Box<dyn Iterator<Item = Self::Solution>>;

    /// Return the optimal neighbor of ```solution```.
    fn find_best_neighbor(&self, solution: Self::Solution) -> Self::Solution {
        let mut winner: Option<Self::Solution> = None;
        for neighbor in self.construct_neighborhood(solution) {
            if let Some(x) = &winner {
                if neighbor.evaluate() < x.evaluate() {
                    winner = Some(neighbor);
                }
            } else {
                winner = Some(neighbor);
            }
        }
        winner.expect("neighborhood was empty")
    }
}

/// A stochastic local search operator.
///
/// An operator of this type does not return a neighborhood, but instead a single randomly drawn neighbor.
pub trait StochasticOperator {
    type Solution;
    /// Return a random neighbor of ```solution```, with respect to the neighborhood induced by this operator, using ```rng``` as source of randomness.
    fn shake(&self, solution: Self::Solution, rng: &mut dyn rand::RngCore) -> Self::Solution;
}

/// Give the next operator based on certain rules.
pub trait OperatorSelector {
    fn select(&self, solution: &dyn Evaluate) -> usize;
}

/// Select operators in a consecutive manner.
///
/// Iterate through all operators, consecutively, starting from the first one. When an improvement is made, the iteration is restarted from the beginning.
pub struct SequentialSelector {
    operator_index: RefCell<usize>,
    n_operators: usize,
    objective_best: RefCell<f32>,
}

/// Select the next operator uniformly at random.
pub struct RandomSelector {
    rng: RefCell<Box<dyn rand::RngCore>>,
    n_operators: usize,
}

/// Solution decorated with some metadata.
///
/// Currently, only the computation time is added to the solution.
pub struct Outcome<T> {
    solution: T,
    duration: std::time::Duration,
}

/// Model of an improvement heuristic based on iterations.
///
/// Models heuristics in the form of:
/// 1. incumbent = initial
/// 2. candidate = ```propose_candidate```(incumbent
/// 3. if ```accept_candidate```(candidate, incumbent)
///     - incumbent = candidate
///     - if incumbent.evaluate() < best_solution.evaluate()
///         - best_solution = incumbent
/// 4. if ```should_terminate```(incumbent)
///     - return best_solution
/// 5. else go back to (2)
pub trait ImprovingHeuristic<Solution> {
    /// Propose a candidate solution given the incumbent.
    ///
    /// In a local search algorithm, the incumbent's neighborhood is searched.
    fn propose_candidate(&self, incumbent: Solution) -> Solution
    where
        Solution: Evaluate;
    /// Test whether the current candidate is accepted as the next incumbent.
    ///
    /// Usually with local search this tests whether the candidate is better than the incumbent.
    /// With simulated annealing, however, acceptance is based on probability.
    fn accept_candidate(&self, candidate: &Solution, incumbent: &Solution) -> bool
    where
        Solution: Evaluate;
    fn should_terminate(&self, incumbent: &Solution) -> bool;
    fn optimize(self, initial: Solution) -> Solution
    where
        Solution: Clone + Evaluate,
        Self: Sized,
    {
        let mut incumbent = initial;
        let mut best_solution = incumbent.clone();
        loop {
            let candidate = self.propose_candidate(incumbent.clone());
            if candidate.evaluate() < best_solution.evaluate() {
                best_solution = candidate.clone();
            }
            if self.accept_candidate(&candidate, &incumbent) {
                incumbent = candidate;
            }
            if self.should_terminate(&incumbent) {
                break;
            }
        }
        best_solution
    }

    /// Runs the [optimize] function and returns an [Outcome], decorated with computation time.
    fn optimize_timed(self, solution: Solution) -> Outcome<Solution>
    where
        Solution: Clone + Evaluate,
        Self: Sized,
    {
        let now = SystemTime::now();
        let solution = self.optimize(solution);
        let duration = now.elapsed().expect("failed to time for duration");
        let outcome = Outcome { duration, solution };
        outcome
    }
}

pub struct SelectorAdaptive<T> {
    options: Vec<T>,
    weights: Vec<f32>,
    index_last_selection: Option<usize>,
    weight_improve_best: f32,
    weight_accept: f32,
    weight_reject: f32,
}

pub enum ProposalEvaluation {
    ImprovedBest,
    Accept,
    Reject,
}

impl<T> SelectorAdaptive<T> {
    pub fn default_parameters(options: Vec<T>) -> Self {
        let n = options.len();
        Self {
            options,
            weights: vec![1.; n],
            index_last_selection: None,
            weight_improve_best: 3.,
            weight_accept: 1.,
            weight_reject: 0.,
        }
    }
    pub fn select(&mut self, rng: &mut dyn rand::RngCore) -> &T {
        let denom: f32 = self.weights.iter().sum();
        let mut sum = 0.;
        let r = rng.gen::<f32>() * denom;

        for i in 0..self.options.len() {
            sum += self.weights[i];
            if r <= sum {
                self.index_last_selection = Some(i);
                return &self.options[i];
            }
        }

        panic!("something went wrong");
    }

    pub fn feedback(&mut self, status: ProposalEvaluation) {
        if let Some(index) = self.index_last_selection {
            let weight = match status {
                ProposalEvaluation::ImprovedBest => self.weight_improve_best,
                ProposalEvaluation::Accept => self.weight_accept,
                ProposalEvaluation::Reject => self.weight_reject,
            };

            self.weights[index] += weight;
        }
    }
}

impl<T> Outcome<T> {
    /// Get the solution which is decorated.
    pub fn solution(&self) -> &T {
        &self.solution
    }

    /// Return the computation time that was needed to get this solution.
    pub fn duration(&self) -> Duration {
        self.duration
    }
}

impl RandomSelector {
    pub fn new<T: rand::RngCore + 'static>(rng: T, n_operators: usize) -> Self {
        Self {
            rng: RefCell::new(Box::new(rng)),
            n_operators,
        }
    }
}

impl OperatorSelector for RandomSelector {
    fn select(&self, _solution: &dyn Evaluate) -> usize {
        self.rng.borrow_mut().gen_range(0..self.n_operators)
    }
}

impl SequentialSelector {
    pub fn new(n: usize) -> Self {
        Self {
            n_operators: n,
            objective_best: RefCell::new(std::f32::INFINITY),
            operator_index: RefCell::new(0),
        }
    }
}

impl OperatorSelector for SequentialSelector {
    fn select(&self, solution: &dyn Evaluate) -> usize {
        let objective = solution.evaluate();
        let k = *self.operator_index.borrow();
        if objective < *self.objective_best.borrow() {
            self.objective_best.replace(objective);
            self.operator_index.borrow_mut().sub_assign(k);
        } else {
            self.operator_index.replace((k + 1) % self.n_operators);
        }

        *self.operator_index.borrow()
    }
}

// todo: make so many tests: for example adaptivity, ...
