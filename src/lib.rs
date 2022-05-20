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
