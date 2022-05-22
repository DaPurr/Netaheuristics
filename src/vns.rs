//! Contains all types relevant to _variable neighborhood search_.
use crate::{
    termination::TerminationCriteria, Evaluate, ImprovingHeuristic, Operator, OperatorSelector,
};

/// Implementation of _variable neighborhood search_ according to [here](https://en.wikipedia.org/wiki/Variable_neighborhood_search).
pub struct VariableNeighborhoodSearch<Solution, Selector: OperatorSelector> {
    operators: Vec<Box<dyn Operator<Solution = Solution>>>,
    selector: Selector,
    terminator: Box<dyn TerminationCriteria<Solution>>,
    // rng: Box<dyn rand::RngCore>,
}

/// Builder pattern to construct a _variable neighborhood search_ heuristic.
pub struct VNSBuilder<Solution, Selector> {
    operators: Vec<Box<dyn Operator<Solution = Solution>>>,
    selector: Option<Selector>,
    terminator: Option<Box<dyn TerminationCriteria<Solution>>>,
    rng: Option<Box<dyn rand::RngCore>>,
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
        return candidate.evaluate() < incumbent.evaluate();
    }

    /// Select operator and get the best neighbor if ```solution```.
    fn propose_candidate(&self, solution: Solution) -> Solution
    where
        Solution: Evaluate,
    {
        let operator_index = self.selector.select(&solution);
        let operator = &self.operators[operator_index];
        operator.find_best_neighbor(solution)
    }

    /// Test whether the termination criteria are fulfilled.
    fn should_terminate(&self, incumbent: &Solution) -> bool {
        self.terminator.terminate(incumbent)
    }
}
