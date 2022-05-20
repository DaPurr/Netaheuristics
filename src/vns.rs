//! Contains all types which are specific to _variable neighborhood search_.
use crate::{termination::TerminationCriteria, Evaluate, Heuristic, Operator};

/// Implementation of _variable neighborhood search_ according to [here](https://en.wikipedia.org/wiki/Variable_neighborhood_search).
pub struct VariableNeighborhoodSearch<Solution, Selector: OperatorSelector> {
    operators: Vec<Box<dyn Operator<Solution>>>,
    selector: Selector,
    terminator: Box<dyn TerminationCriteria<Solution>>,
    rng: Box<dyn rand::RngCore>,
}

/// Select operators consecutively.
///
/// Iterate through all operators, starting from the first one. When an improvement is made, the iteration is restarted from the beginning.
pub struct SequentialSelector {
    operator_index: usize,
}

/// Types implementing this trait are able to select the next operator.
pub trait OperatorSelector {
    fn initial_operator(&self) -> usize;
    fn select_operator(&mut self, did_improve: bool) -> usize;
}

/// Builder pattern to construct a _variable neighborhood search_ heuristic.
pub struct VNSBuilder<Solution, Selector> {
    operators: Vec<Box<dyn Operator<Solution>>>,
    selector: Option<Selector>,
    terminator: Option<Box<dyn TerminationCriteria<Solution>>>,
    rng: Option<Box<dyn rand::RngCore>>,
}

impl Default for SequentialSelector {
    fn default() -> Self {
        Self { operator_index: 0 }
    }
}

impl OperatorSelector for SequentialSelector {
    /// Choose the first operator initially.
    fn initial_operator(&self) -> usize {
        0
    }

    /// Select the next operator, as initially specified by the user.
    fn select_operator(&mut self, did_improve: bool) -> usize {
        if did_improve {
            self.operator_index = 0;
        } else {
            self.operator_index += 1;
        }

        self.operator_index
    }
}

impl<'a, Solution, Selector: OperatorSelector> VNSBuilder<Solution, Selector> {
    /// Add an operator.
    pub fn operator<T: 'static + Operator<Solution>>(mut self, operator: T) -> Self {
        let operator: Box<dyn Operator<Solution>> = Box::new(operator);
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
        let rng: Box<dyn rand::RngCore> = Box::new(rand::thread_rng());
        VariableNeighborhoodSearch {
            operators: self.operators,
            selector: self.selector.expect("Did not specify an operator selector"),
            terminator: self
                .terminator
                .expect("Did not specify termination criteria"),
            rng: self.rng.unwrap_or(rng),
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

impl<Solution: Evaluate + Clone, Selector: OperatorSelector> Heuristic
    for VariableNeighborhoodSearch<Solution, Selector>
{
    /// Implementation of the _variable neighborhood search_ routine.
    ///
    /// Starting with an initial solution, the following steps are repeated as long as the termination criteria say so:
    /// 1. Shake the incumbent with respect to the chosen operator (starting with the initial one);
    /// 2. Select the best solution in the neighborhood;
    /// 3. Update the incumbent, if necessary;
    /// 4. Select the next operator;
    /// 5. Evaluate the termination criteria.
    type Solution = Solution;
    fn optimize(mut self, initial_solution: Self::Solution) -> Self::Solution {
        // init
        let terminator = self.terminator;
        let mut selector = self.selector;
        let mut operator_index = selector.initial_operator();
        let mut incumbent = initial_solution.clone();
        let ref mut rng = self.rng;

        loop {
            // init
            let mut did_improve = false;
            let ref operator = self.operators[operator_index];

            let shaken = operator.shake(incumbent.clone(), rng);
            let best_neighbor = operator.find_best_neighbor(shaken);
            if best_neighbor.evaluate() > incumbent.evaluate() {
                incumbent = best_neighbor;
                did_improve = true;
            }

            // select operator
            operator_index = selector.select_operator(did_improve);
            // check if operator index is valid
            if operator_index >= self.operators.len() {
                break;
            }

            // test termination criteria
            if terminator.terminate(&incumbent) {
                break;
            }
        }

        incumbent
    }
}
