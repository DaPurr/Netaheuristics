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
use std::time::{Duration, SystemTime};

pub mod algorithms;
pub mod selectors;
pub mod termination;
#[cfg(test)]
mod test;

/// Evaluate the quality of a solution.
pub trait Evaluate {
    fn evaluate(&self) -> f32;
}

/// A local search operator returns the neighborhood of its argument.
pub trait Operator {
    type Solution: Evaluate;
    /// Construct the neighborhood of ```solution```.
    #[allow(unused_variables)]
    fn construct_neighborhood(
        &self,
        solution: Self::Solution,
    ) -> Box<dyn Iterator<Item = Self::Solution>> {
        todo!()
    }

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

    #[allow(unused_variables)]
    fn shake(&self, solution: Self::Solution, rng: &mut dyn rand::RngCore) -> Self::Solution {
        todo!()
    }
}

/// Give the next operator based on certain rules.
#[allow(unused_variables)]
pub trait OperatorSelector<Solution> {
    fn select(&self, solution: &dyn Evaluate) -> &dyn Operator<Solution = Solution>;

    fn feedback(&self, status: ProposalEvaluation) {}
}

/// Solution decorated with some metadata
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
                self.callback_candidate_improved_best(&candidate, &incumbent);
                best_solution = candidate.clone();
            }
            if self.accept_candidate(&candidate, &incumbent) {
                self.callback_candidate_accepted(&candidate, &incumbent);
                incumbent = candidate;
            } else {
                self.callback_candidate_rejected(&candidate, &incumbent);
            }
            if self.should_terminate(&incumbent) {
                break;
            }
        }
        best_solution
    }

    #[allow(unused_variables)]
    fn callback_candidate_improved_best(&self, candidate: &Solution, incumbent: &Solution) {}
    #[allow(unused_variables)]
    fn callback_candidate_accepted(&self, candidate: &Solution, incumbent: &Solution) {}
    #[allow(unused_variables)]
    fn callback_candidate_rejected(&self, candidate: &Solution, incumbent: &Solution) {}

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

/// Evaluation of a proposed candidate
pub enum ProposalEvaluation {
    /// Candidate improved the incumbent
    ImprovedBest,
    /// Candidate was accepted
    Accept,
    /// Candidate was rejected
    Reject,
}

impl<T> Outcome<T> {
    pub fn new(solution: T, duration: Duration) -> Self {
        Self { solution, duration }
    }

    /// Get the solution which is decorated.
    pub fn solution(&self) -> &T {
        &self.solution
    }

    /// Return the computation time that was needed to get this solution.
    pub fn duration(&self) -> Duration {
        self.duration
    }
}

// todo: clean up code / refactor
// todo: implement adaptivity for SA
