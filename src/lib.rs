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

use rand::{Rng, RngCore};

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

/// A stochastic local search operator.
///
/// An operator of this type does not return a neighborhood, but instead a single randomly drawn neighbor.
pub trait StochasticOperator {
    type Solution;
    /// Return a random neighbor of ```solution```, with respect to the neighborhood induced by this operator, using ```rng``` as source of randomness.
    fn shake(&self, solution: Self::Solution, rng: &mut dyn rand::RngCore) -> Self::Solution;
}

/// Give the next operator based on certain rules.
#[allow(unused_variables)]
pub trait OperatorSelector<Solution> {
    fn select(&self, solution: &dyn Evaluate) -> &dyn Operator<Solution = Solution>;

    fn feedback(&self, status: ProposalEvaluation) {}
}

/// Select operators in a consecutive manner.
///
/// Iterate through all operators, consecutively, starting from the first one. When an improvement is made, the iteration is restarted from the beginning.
pub struct SequentialSelector<Solution> {
    operators: Vec<Box<dyn Operator<Solution = Solution>>>,
    operator_index: RefCell<usize>,
    objective_best: RefCell<f32>,
}

/// Select the next operator uniformly at random.
pub struct RandomSelector<Solution> {
    operators: Vec<Box<dyn Operator<Solution = Solution>>>,
    rng: RefCell<Box<dyn RngCore>>,
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

pub struct SelectorAdaptive<T> {
    rng: RefCell<Box<dyn rand::RngCore>>,
    options: Vec<T>,
    weights: Vec<f32>,
    decay: f32,
    index_last_selection: RefCell<Option<usize>>,
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
    pub fn default_weights<Rng: rand::RngCore + 'static>(
        options: Vec<T>,
        decay: f32,
        rng: Rng,
    ) -> Self {
        let n = options.len();
        Self {
            rng: RefCell::new(Box::new(rng)),
            options,
            decay,
            weights: vec![1.; n],
            index_last_selection: RefCell::new(None),
            weight_improve_best: 3.,
            weight_accept: 1.,
            weight_reject: 0.,
        }
    }

    pub fn feedback(&mut self, status: ProposalEvaluation) {
        if let Some(index) = self.index_last_selection.borrow().as_ref() {
            let index = *index;
            let weight = match status {
                ProposalEvaluation::ImprovedBest => self.weight_improve_best,
                ProposalEvaluation::Accept => self.weight_accept,
                ProposalEvaluation::Reject => self.weight_reject,
            };

            self.weights[index] = (1. - self.decay) * self.weights[index] + self.decay * weight;
        }
    }
}

impl<Solution> OperatorSelector<Solution>
    for SelectorAdaptive<Box<dyn Operator<Solution = Solution>>>
{
    fn select(&self, _solution: &dyn Evaluate) -> &dyn Operator<Solution = Solution> {
        let ref rng = self.rng;
        let denom: f32 = self.weights.iter().sum();
        let mut sum = 0.;
        let r = rng.borrow_mut().gen::<f32>() * denom;
        for i in 0..self.options.len() {
            sum += self.weights[i];
            if r <= sum {
                self.index_last_selection.replace(Some(i));
                return self.options[i].as_ref();
            }
        }

        panic!("something went wrong");
    }
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

impl<Solution> RandomSelector<Solution> {
    pub fn new<T: rand::RngCore + 'static>(
        operators: Vec<Box<dyn Operator<Solution = Solution>>>,
        rng: T,
    ) -> Self {
        Self {
            operators,
            rng: RefCell::new(Box::new(rng)),
        }
    }
}

impl<Solution> OperatorSelector<Solution> for RandomSelector<Solution> {
    fn select(&self, _solution: &dyn Evaluate) -> &dyn Operator<Solution = Solution> {
        let index = self.rng.borrow_mut().gen_range(0..self.operators.len());
        self.operators[index].as_ref()
    }
}

impl<Solution> SequentialSelector<Solution> {
    pub fn new(operators: Vec<Box<dyn Operator<Solution = Solution>>>) -> Self {
        Self {
            operators,
            objective_best: RefCell::new(std::f32::INFINITY),
            operator_index: RefCell::new(0),
        }
    }
}

impl<Solution> OperatorSelector<Solution> for SequentialSelector<Solution> {
    fn select(&self, solution: &dyn Evaluate) -> &dyn Operator<Solution = Solution> {
        let objective = solution.evaluate();
        let k = *self.operator_index.borrow();
        if objective < *self.objective_best.borrow() {
            self.objective_best.replace(objective);
            self.operator_index.borrow_mut().sub_assign(k);
        } else {
            self.operator_index.replace((k + 1) % self.operators.len());
        }

        let index = *self.operator_index.borrow();
        self.operators[index].as_ref()
    }
}

// todo: clean up code / refactor
// todo: implement adaptivity for SA
