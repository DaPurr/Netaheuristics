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

pub mod termination;
#[cfg(test)]
mod test;
pub mod vns;

/// Evaluate the quality of a solution.
pub trait Evaluate {
    fn evaluate(&self) -> f32;
}

/// An local search operator returns the neighborhood of its argument.
pub trait Operator<Solution> {
    /// Construct the neighborhood ```solution```.
    fn construct_neighborhood(&self, solution: Solution) -> Box<dyn Iterator<Item = Solution>>;

    /// Return a random neighbor of ```solution```, with respect to the neighborhood induced by this operator, using ```rng``` as source of randomness.
    #[allow(unused_variables)]
    fn shake(&self, solution: Solution, rng: &mut dyn rand::RngCore) -> Solution {
        solution
    }

    /// Return the optimal neighbor of ```solution```.
    fn find_best_neighbor(&self, solution: Solution) -> Solution
    where
        Solution: Evaluate,
    {
        let mut winner: Option<Solution> = None;
        for neighbor in self.construct_neighborhood(solution) {
            if let Some(x) = &winner {
                if neighbor.evaluate() > x.evaluate() {
                    winner = Some(neighbor);
                }
            } else {
                winner = Some(neighbor);
            }
        }
        winner.expect("neighborhood was empty")
    }
}

pub trait Heuristic {
    type Solution: Evaluate + Clone;
    fn optimize(self, solution: Self::Solution) -> Self::Solution;
}
