use crate::{Evaluate, LocalSearchHeuristic, Operator};

pub struct VariableNeighborhoodSearch<'a, Solution, Callbacks> {
    operators: Vec<Box<dyn Operator<'a, Solution> + 'a>>,
    callbacks: Callbacks,
}

pub trait VNSCallbacks<Solution>
where
    Solution: Evaluate,
{
    fn select_operator(&mut self) -> usize;
    fn propose_candidate(&mut self, candidate: &Solution) -> bool;
    fn should_terminate(&mut self) -> bool;
}

pub struct BasicVNSCallbacks {
    objective_best: Option<f32>,
    iterations_no_improvement: usize,
    iteration: usize,
    iterations_max: usize,
    n_operators: usize,
    index_operator: usize,
    did_improve: bool,
}

pub struct VNSBuilder<'a, Solution, Callbacks> {
    operators: Vec<Box<dyn Operator<'a, Solution> + 'a>>,
    callbacks: Option<Callbacks>,
}

pub trait OperatorSelector {
    fn select_operator(&self) -> usize;
}

pub trait TerminationCriteria {
    fn should_terminate(&self) -> bool;
}

pub struct SimpleOperatorSelector;

pub struct SimpleTerminationCriteria;

impl OperatorSelector for SimpleOperatorSelector {
    fn select_operator(&self) -> usize {
        todo!()
    }
}

impl TerminationCriteria for SimpleTerminationCriteria {
    fn should_terminate(&self) -> bool {
        todo!()
    }
}

impl<'a, Solution, Callbacks> VNSBuilder<'a, Solution, Callbacks> {
    pub fn operator<T: Operator<'a, Solution> + 'a>(mut self, operator: T) -> Self {
        let operator: Box<dyn Operator<Solution>> = Box::new(operator);
        self.operators.push(operator);
        self
    }

    pub fn callbacks(mut self, callbacks: Callbacks) -> Self {
        self.callbacks = Some(callbacks);
        self
    }

    pub fn build(self) -> VariableNeighborhoodSearch<'a, Solution, Callbacks> {
        VariableNeighborhoodSearch {
            operators: self.operators,
            callbacks: self.callbacks.unwrap(),
        }
    }
}

impl BasicVNSCallbacks {
    pub fn new(n_operators: usize) -> Self {
        Self {
            objective_best: None,
            iteration: 0,
            iterations_no_improvement: 0,
            iterations_max: std::usize::MAX,
            n_operators,
            index_operator: 0,
            did_improve: false,
        }
    }

    pub fn objective_best(&self) -> Option<f32> {
        self.objective_best
    }

    pub fn iterations_no_improvement(&self) -> usize {
        self.iterations_no_improvement
    }

    pub fn set_objective_best(&mut self, objective: f32) {
        self.objective_best = Some(objective)
    }
}

impl<Solution: Evaluate + Clone> VNSCallbacks<Solution> for BasicVNSCallbacks {
    fn select_operator(&mut self) -> usize {
        self.index_operator
    }

    fn propose_candidate(&mut self, candidate: &Solution) -> bool {
        let objective_new = candidate.evaluate();
        match self.objective_best() {
            Some(objective_old) => {
                if objective_new > objective_old {
                    self.did_improve = true;
                    self.objective_best = Some(objective_new);
                    self.index_operator = 0;
                    true
                } else {
                    self.did_improve = false;
                    false
                }
            }
            None => {
                self.objective_best = Some(objective_new);
                self.did_improve = true;
                true
            }
        }
    }

    fn should_terminate(&mut self) -> bool {
        self.iteration += 1;
        if self.iteration == self.iterations_max {
            return true;
        }

        if !self.did_improve {
            self.index_operator += 1;
            self.iterations_no_improvement += 1;
        }

        if self.iterations_no_improvement == self.n_operators {
            return true;
        }

        false
    }
}

impl<'a, Solution, Callbacks> VariableNeighborhoodSearch<'a, Solution, Callbacks> {
    pub fn builder() -> VNSBuilder<'a, Solution, Callbacks> {
        VNSBuilder {
            callbacks: None,
            operators: vec![],
        }
    }
}

impl<'a, Solution, Callbacks> LocalSearchHeuristic<Solution>
    for VariableNeighborhoodSearch<'a, Solution, Callbacks>
where
    Solution: Evaluate + Clone,
    Callbacks: VNSCallbacks<Solution>,
{
    fn optimize(self, initial_solution: Solution) -> Solution {
        // init
        let mut best_solution = initial_solution.clone();
        let mut callbacks = self.callbacks;

        loop {
            let index_operator = callbacks.select_operator();
            let ref operator = self.operators[index_operator];
            for neighbor in operator.construct_neighborhood(best_solution.clone()) {
                if callbacks.propose_candidate(&neighbor) {
                    best_solution = neighbor;
                }
            }

            if callbacks.should_terminate() {
                break;
            }
        }

        best_solution
    }
}
