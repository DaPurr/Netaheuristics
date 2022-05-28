//! Contains all types relevant to _variable neighborhood search_.
use std::cell::RefCell;

use crate::{
    termination::TerminationCriteria, Evaluate, ImprovingHeuristic, Operator, OperatorSelector,
    ProposalEvaluation, SelectorAdaptive,
};

/// Implementation of _variable neighborhood search_ according to [here](https://en.wikipedia.org/wiki/Variable_neighborhood_search).
pub struct VariableNeighborhoodSearch<Solution, Selector: OperatorSelector> {
    operators: Vec<Box<dyn Operator<Solution = Solution>>>,
    selector: Selector,
    terminator: Box<dyn TerminationCriteria<Solution>>,
}

pub struct AdaptiveVariableNeighborhoodSearch<Solution> {
    selector: RefCell<SelectorAdaptive<Box<dyn Operator<Solution = Solution>>>>,
    terminator: Box<dyn TerminationCriteria<Solution>>,
    best_objective: f32,
    rng: RefCell<Box<dyn rand::RngCore>>,
}

/// Builder pattern to construct a _variable neighborhood search_ heuristic.
pub struct VNSBuilder<Solution, Selector> {
    operators: Vec<Box<dyn Operator<Solution = Solution>>>,
    selector: Option<Selector>,
    terminator: Option<Box<dyn TerminationCriteria<Solution>>>,
    rng: Option<Box<dyn rand::RngCore>>,
}

impl<Solution> AdaptiveVariableNeighborhoodSearch<Solution> {
    pub fn new<
        Terminator: TerminationCriteria<Solution> + 'static,
        Rng: rand::RngCore + 'static,
    >(
        initial: &dyn Evaluate,
        operators: Vec<Box<dyn Operator<Solution = Solution>>>,
        decay: f32,
        terminator: Terminator,
        rng: Rng,
    ) -> Self {
        let selector = SelectorAdaptive::default_parameters(operators, decay);
        Self {
            best_objective: initial.evaluate(),
            selector: RefCell::new(selector),
            terminator: Box::new(terminator),
            rng: RefCell::new(Box::new(rng)),
        }
    }
}

impl<Solution> ImprovingHeuristic<Solution> for AdaptiveVariableNeighborhoodSearch<Solution> {
    fn accept_candidate(&self, candidate: &Solution, incumbent: &Solution) -> bool
    where
        Solution: Evaluate,
    {
        accept_candidate_if_better(candidate, incumbent)
    }

    fn should_terminate(&self, incumbent: &Solution) -> bool {
        self.terminator.terminate(incumbent)
    }

    fn propose_candidate(&self, incumbent: Solution) -> Solution
    where
        Solution: Evaluate,
    {
        let incumbent_objective = incumbent.evaluate();
        let mut selector = self.selector.borrow_mut();
        let operator = selector.select(self.rng.borrow_mut().as_mut());
        let candidate = operator.find_best_neighbor(incumbent);
        let candidate_objective = candidate.evaluate();

        if candidate_objective < self.best_objective {
            selector.feedback(ProposalEvaluation::ImprovedBest)
        } else if candidate_objective < incumbent_objective {
            selector.feedback(ProposalEvaluation::Accept)
        } else {
            selector.feedback(ProposalEvaluation::Reject)
        }

        candidate
    }
}

impl<'a, Solution, Selector: OperatorSelector> VNSBuilder<Solution, Selector> {
    /// Add an operator.
    pub fn operator<T: 'static + Operator<Solution = Solution>>(mut self, operator: T) -> Self {
        let operator: Box<dyn Operator<Solution = Solution>> = Box::new(operator);
        self.operators.push(operator);
        self
    }

    /// Specify the operator selector to be used.
    pub fn selector(mut self, selector: Selector) -> Self {
        self.selector = Some(selector);
        self
    }

    /// Specify the termination criteria.
    pub fn terminator(mut self, terminator: Box<dyn TerminationCriteria<Solution>>) -> Self {
        self.terminator = Some(terminator);
        self
    }

    /// Specify the source of randomness.
    pub fn rng<T: rand::RngCore + 'static>(mut self, rng: T) -> Self {
        self.rng = Some(Box::new(rng));
        self
    }

    /// Construct the specified heuristic.
    pub fn build(self) -> VariableNeighborhoodSearch<Solution, Selector> {
        VariableNeighborhoodSearch {
            operators: self.operators,
            selector: self.selector.expect("Did not specify an operator selector"),
            terminator: self
                .terminator
                .expect("Did not specify termination criteria"),
        }
    }
}

impl<'a, Solution, Selector: OperatorSelector> VariableNeighborhoodSearch<Solution, Selector> {
    /// Return a builder to simplify the specification.
    pub fn builder() -> VNSBuilder<Solution, Selector> {
        VNSBuilder {
            selector: None,
            operators: vec![],
            rng: None,
            terminator: None,
        }
    }
}

impl<Solution, Selector> ImprovingHeuristic<Solution>
    for VariableNeighborhoodSearch<Solution, Selector>
where
    Selector: OperatorSelector,
{
    /// Accept iff candidate is better than the incumbent.
    fn accept_candidate(&self, candidate: &Solution, incumbent: &Solution) -> bool
    where
        Solution: Evaluate,
    {
        accept_candidate_if_better(candidate, incumbent)
    }

    /// Select operator and get the best neighbor if ```solution```.
    fn propose_candidate(&self, solution: Solution) -> Solution
    where
        Solution: Evaluate,
    {
        let operator_index = self.selector.select(&solution);
        println!("{}", operator_index);
        let operator = &self.operators[operator_index];
        operator.find_best_neighbor(solution)
    }

    /// Test whether the termination criteria are fulfilled.
    fn should_terminate(&self, incumbent: &Solution) -> bool {
        self.terminator.terminate(incumbent)
    }
}

fn accept_candidate_if_better(candidate: &dyn Evaluate, incumbent: &dyn Evaluate) -> bool {
    return candidate.evaluate() < incumbent.evaluate();
}
