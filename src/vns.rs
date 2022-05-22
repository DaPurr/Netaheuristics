//! Contains all types which are specific to _variable neighborhood search_.
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

pub trait StochasticOperator {
    type Solution;
    fn shake(&self, solution: Self::Solution, rng: &mut dyn rand::RngCore) -> Self::Solution;
}

impl<Solution> StochasticOperator for dyn Operator<Solution = Solution>
where
    Solution: Clone,
{
    type Solution = Solution;
    fn shake(&self, solution: Self::Solution, _rng: &mut dyn rand::RngCore) -> Self::Solution {
        solution.clone()
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

// impl<Solution: Evaluate + Clone + Debug, Selector: OperatorSelector> Heuristic<Solution>
//     for VariableNeighborhoodSearch<Solution, Selector>
// {
//     /// Implementation of the _variable neighborhood search_ routine.
//     ///
//     /// Starting with an initial solution, the following steps are repeated as long as the termination criteria say so:
//     /// 1. Shake the incumbent with respect to the chosen operator (starting with the initial one);
//     /// 2. Select the best solution in the neighborhood;
//     /// 3. Update the incumbent, if necessary;
//     /// 4. Select the next operator;
//     /// 5. Evaluate the termination criteria.
//     fn optimize(mut self, initial_solution: Solution) -> Solution {
//         // init
//         let mut incumbent = initial_solution;
//         let terminator = self.terminator;
//         let selector = self.selector;
//         let mut operator_index = selector.select(&incumbent);
//         let ref mut rng = self.rng;

//         loop {
//             // init
//             let ref operator = self.operators[operator_index];

//             let shaken = operator.shake(incumbent.clone(), rng);
//             let best_neighbor = operator.find_best_neighbor(shaken);
//             if best_neighbor.evaluate() < incumbent.evaluate() {
//                 incumbent = best_neighbor;
//             }

//             // select operator
//             operator_index = selector.select(&incumbent);
//             // check if operator index is valid
//             if operator_index >= self.operators.len() {
//                 break;
//             }

//             // test termination criteria
//             if terminator.terminate(&incumbent) {
//                 break;
//             }
//         }

//         incumbent
//     }
// }

impl<Solution, Selector> ImprovingHeuristic<Solution>
    for VariableNeighborhoodSearch<Solution, Selector>
where
    Selector: OperatorSelector,
{
    fn accept_candidate(&self, candidate: &Solution, incumbent: &Solution) -> bool
    where
        Solution: Evaluate,
    {
        return candidate.evaluate() < incumbent.evaluate();
    }

    fn propose_candidate(&self, solution: Solution) -> Solution
    where
        Solution: Evaluate,
    {
        let operator_index = self.selector.select(&solution);
        let operator = &self.operators[operator_index];
        operator.find_best_neighbor(solution)
    }

    fn should_terminate(&self, incumbent: &Solution) -> bool {
        self.terminator.terminate(incumbent)
    }
}
