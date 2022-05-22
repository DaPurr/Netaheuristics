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

use std::{cell::RefCell, ops::SubAssign};

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

/// An local search operator returns the neighborhood of its argument.
pub trait Operator {
    type Solution: Evaluate;
    /// Construct the neighborhood ```solution```.
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

pub trait StochasticOperator {
    type Solution;
    /// Return a random neighbor of ```solution```, with respect to the neighborhood induced by this operator, using ```rng``` as source of randomness.
    fn shake(&self, solution: Self::Solution, rng: &mut dyn rand::RngCore) -> Self::Solution;
}

pub trait Heuristic {
    type Solution: Evaluate;
    fn optimize(self, solution: Self::Solution) -> Self::Solution;
}

/// Types implementing this trait are able to select the next operator.
pub trait OperatorSelector {
    fn select(&self, solution: &dyn Evaluate) -> usize;
}

/// Select operators consecutively.
///
/// Iterate through all operators, starting from the first one. When an improvement is made, the iteration is restarted from the beginning.
pub struct SequentialSelector {
    operator_index: RefCell<usize>,
    n_operators: usize,
    objective_best: RefCell<f32>,
}

pub struct RandomSelector {
    rng: RefCell<Box<dyn rand::RngCore>>,
    n_operators: usize,
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
    /// Select the next operator, as initially specified by the user.
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
