use crate::{Evaluate, LocalSearchHeuristic, Operator};

pub struct VariableNeighborhoodSearch<
    Solution,
    Selector: OperatorSelector,
    Terminator: TerminationCriteria,
> {
    operators: Vec<Box<dyn Operator<Solution>>>,
    selector: Selector,
    terminator: Terminator,
    rng: Box<dyn rand::RngCore>,
}

pub struct SequentialSelector {
    operator_index: usize,
}

pub trait OperatorSelector {
    fn initial_operator(&self) -> usize;
    fn select_operator(&mut self, did_improve: bool) -> usize;
}

pub trait TerminationCriteria {
    fn terminate(&self) -> bool;
}

pub struct VNSBuilder<Solution, Selector, Terminator> {
    operators: Vec<Box<dyn Operator<Solution>>>,
    selector: Option<Selector>,
    terminator: Option<Terminator>,
    rng: Option<Box<dyn rand::RngCore>>,
}

pub struct TerminationCriteriaDefault {
    n_iterations_max: usize,
    iteration: usize,
}

impl TerminationCriteriaDefault {
    pub fn new(n_iterations_max: usize) -> Self {
        Self {
            n_iterations_max,
            iteration: 0,
        }
    }
}

impl TerminationCriteria for TerminationCriteriaDefault {
    fn terminate(&self) -> bool {
        self.iteration >= self.n_iterations_max
    }
}

impl Default for SequentialSelector {
    fn default() -> Self {
        Self { operator_index: 0 }
    }
}

impl OperatorSelector for SequentialSelector {
    fn initial_operator(&self) -> usize {
        0
    }

    fn select_operator(&mut self, did_improve: bool) -> usize {
        if did_improve {
            self.operator_index = 0;
        } else {
            self.operator_index += 1;
        }

        self.operator_index
    }
}

impl<'a, Solution, Selector: OperatorSelector, Terminator: TerminationCriteria>
    VNSBuilder<Solution, Selector, Terminator>
{
    pub fn operator<T: 'static + Operator<Solution>>(mut self, operator: T) -> Self {
        let operator: Box<dyn Operator<Solution>> = Box::new(operator);
        self.operators.push(operator);
        self
    }

    pub fn selector(mut self, selector: Selector) -> Self {
        self.selector = Some(selector);
        self
    }

    pub fn terminator(mut self, termination_criteria: Terminator) -> Self {
        self.terminator = Some(termination_criteria);
        self
    }

    pub fn rng<T: rand::RngCore + 'static>(mut self, rng: T) -> Self {
        self.rng = Some(Box::new(rng));
        self
    }

    pub fn build(self) -> VariableNeighborhoodSearch<Solution, Selector, Terminator> {
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

impl<'a, Solution, Selector: OperatorSelector, Terminator: TerminationCriteria>
    VariableNeighborhoodSearch<Solution, Selector, Terminator>
{
    pub fn builder() -> VNSBuilder<Solution, Selector, Terminator> {
        VNSBuilder {
            selector: None,
            terminator: None,
            operators: vec![],
            rng: None,
        }
    }
}

impl<
        'a,
        Solution: Evaluate + Clone,
        Selector: OperatorSelector,
        Terminator: TerminationCriteria,
    > LocalSearchHeuristic for VariableNeighborhoodSearch<Solution, Selector, Terminator>
{
    type Solution = Solution;
    fn optimize(mut self, initial_solution: Self::Solution) -> Self::Solution {
        // init
        let terminator = self.terminator;
        let mut selector = self.selector;
        let mut operator_index = selector.initial_operator();
        let mut incumbent = initial_solution.clone();
        let ref mut rng = self.rng;

        loop {
            let mut did_improve = false;
            let ref operator = self.operators[operator_index];
            incumbent = operator.shake(incumbent, rng);
            for neighbor in operator.construct_neighborhood(incumbent.clone()) {
                let objective_incumbent = incumbent.evaluate();
                let objective_candidate = neighbor.evaluate();
                if objective_candidate > objective_incumbent {
                    incumbent = neighbor;
                    did_improve = true;
                }
            }

            // select operator
            operator_index = selector.select_operator(did_improve);
            // check if operator index is valid
            if operator_index >= self.operators.len() {
                break;
            }

            // test termination criteria
            if terminator.terminate() {
                break;
            }
        }

        incumbent
    }
}
