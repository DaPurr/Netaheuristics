//! Select the next operator to be used
use std::{cell::RefCell, ops::SubAssign};

use rand::Rng;

use crate::{Evaluate, Operator, ProposalEvaluation};

/// Give the next operator based on certain rules.
#[allow(unused_variables)]
pub trait OperatorSelector<Solution> {
    /// Select the next operator based on the rules specified by the implementing type
    fn select(&self, solution: &dyn Evaluate) -> &dyn Operator<Solution = Solution>;

    /// Give feedback on the last selected operator
    fn feedback(&self, status: ProposalEvaluation) {}
}

/// Select operators in a consecutive manner
///
/// Iterate through all operators, consecutively, starting from the first one. When an improvement is made, the iteration is restarted from the beginning.
pub struct SequentialSelector<Solution> {
    operators: Vec<Box<dyn Operator<Solution = Solution>>>,
    operator_index: RefCell<usize>,
    objective_best: RefCell<f32>,
}

/// Select the next operator uniformly at random
pub struct RandomSelector<Solution> {
    operators: Vec<Box<dyn Operator<Solution = Solution>>>,
    rng: RefCell<Box<dyn rand::RngCore>>,
}

/// Select the next operator adaptively
///
/// Learn when which operator is performing well by
/// receiving feedback.
pub struct AdaptiveSelector<Solution> {
    rng: RefCell<Box<dyn rand::RngCore>>,
    options: Vec<Box<dyn Operator<Solution = Solution>>>,
    weights: Vec<f32>,
    decay: f32,
    index_last_selection: RefCell<Option<usize>>,
    weight_improve_best: f32,
    weight_accept: f32,
    weight_reject: f32,
}

impl<Solution> AdaptiveSelector<Solution> {
    /// Create an [AdaptiveSelector] with default weights. They are:
    /// - Best solution improved: 3
    /// - Accepted candidate: 1
    /// - Rejected cadidate: 0
    pub fn default_weights<Rng: rand::RngCore + 'static>(decay: f32, rng: Rng) -> Self {
        Self {
            rng: RefCell::new(Box::new(rng)),
            decay,
            options: vec![],
            weights: vec![],
            index_last_selection: RefCell::new(None),
            weight_improve_best: 3.,
            weight_accept: 1.,
            weight_reject: 0.,
        }
    }

    /// Create an [AdaptiveSelector] with custom weights
    pub fn custom_weights<Rng: rand::RngCore + 'static>(
        decay: f32,
        weight_improve_best: f32,
        weight_accept: f32,
        weight_reject: f32,
        rng: Rng,
    ) -> Self {
        Self {
            rng: RefCell::new(Box::new(rng)),
            decay,
            options: vec![],
            weights: vec![],
            index_last_selection: RefCell::new(None),
            weight_improve_best,
            weight_accept,
            weight_reject,
        }
    }

    /// Give feedback on the last chosen operator based on the last proposed candidate.
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

    /// Add operator to the operator pool
    pub fn operator<T: Operator<Solution = Solution> + 'static>(mut self, option: T) -> Self {
        self.options.push(Box::new(option));
        self.weights.push(1.);
        self
    }
}

impl<Solution> OperatorSelector<Solution> for AdaptiveSelector<Solution> {
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

        panic!("Could not select operator");
    }
}

impl<Solution> RandomSelector<Solution> {
    pub fn new<T: rand::RngCore + 'static>(rng: T) -> Self {
        Self {
            operators: vec![],
            rng: RefCell::new(Box::new(rng)),
        }
    }

    pub fn option<T: Operator<Solution = Solution> + 'static>(mut self, option: T) -> Self {
        self.operators.push(Box::new(option));
        self
    }
}

impl<Solution> OperatorSelector<Solution> for RandomSelector<Solution> {
    fn select(&self, _solution: &dyn Evaluate) -> &dyn Operator<Solution = Solution> {
        let index = self.rng.borrow_mut().gen_range(0..self.operators.len());
        self.operators[index].as_ref()
    }
}

impl<Solution> SequentialSelector<Solution> {
    pub fn new() -> Self {
        Self {
            operators: vec![],
            objective_best: RefCell::new(std::f32::INFINITY),
            operator_index: RefCell::new(0),
        }
    }

    pub fn option<T: Operator<Solution = Solution> + 'static>(mut self, option: T) -> Self {
        self.operators.push(Box::new(option));
        self
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

#[cfg(test)]
mod tests {
    use crate::test::*;
    use assert_approx_eq::assert_approx_eq;
    use rand::SeedableRng;

    use crate::{selectors::AdaptiveSelector, ProposalEvaluation};

    #[test]
    fn adaptivity_core() {
        let rng = rand::rngs::StdRng::seed_from_u64(0);
        let op1 = NeighborSwap::new(&[1., 2., 3.]);
        let op2 = NeighborSwap::new(&[1., 2., 3.]);
        let op3 = NeighborSwap::new(&[1., 2., 3.]);
        let mut selector = AdaptiveSelector::default_weights(1., rng)
            .operator(op1)
            .operator(op2)
            .operator(op3);
        assert_approx_eq!(selector.weights[0], 1.);
        assert_approx_eq!(selector.weights[1], 1.);
        assert_approx_eq!(selector.weights[2], 1.);

        selector.index_last_selection.replace(Some(0));
        selector.feedback(ProposalEvaluation::ImprovedBest);
        assert_approx_eq!(selector.weights[0], 3.);
        assert_approx_eq!(selector.weights[1], 1.);
        assert_approx_eq!(selector.weights[2], 1.);

        selector.index_last_selection.replace(Some(2));
        selector.feedback(ProposalEvaluation::Accept);
        assert_approx_eq!(selector.weights[0], 3.);
        assert_approx_eq!(selector.weights[1], 1.);
        assert_approx_eq!(selector.weights[2], 1.);
    }
}
