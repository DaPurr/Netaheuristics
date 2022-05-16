use std::cell::RefCell;

use crate::{Evaluate, LocalSearchHeuristic, Operator};

pub struct VariableNeighborhoodSearch<'a, Solution, Callbacks> {
    operators: Vec<Box<dyn Operator<'a, Solution> + 'a>>,
    callbacks: Option<RefCell<Callbacks>>,
}

pub trait VNSCallbacks<Solution>
where
    Solution: Evaluate,
{
    #[allow(unused_variables)]
    fn propose_candidate(&mut self, candidate: &Solution) {}

    fn should_terminate(&mut self) -> bool {
        false
    }
}

pub struct BasicVNSCallbacks {
    iteration: usize,
    iteration_max: usize,
    iteration_no_improvement: usize,
    n_operators: usize,
    best_objective: Option<f32>,
}

impl<Solution: Evaluate + Clone> VNSCallbacks<Solution> for BasicVNSCallbacks {
    fn propose_candidate(&mut self, candidate: &Solution) {
        let objective_new = candidate.evaluate();
        match &self.best_objective {
            Some(objective_old) => {
                if objective_new > *objective_old {
                    self.best_objective = Some(objective_new);
                    self.iteration_no_improvement = 0;
                } else {
                    self.iteration_no_improvement += 1;
                }
            }
            None => {
                self.best_objective = Some(objective_new);
            }
        }
    }

    fn should_terminate(&mut self) -> bool {
        self.iteration += 1;
        if self.iteration >= self.iteration_max {
            return true;
        }

        if self.iteration_no_improvement >= self.n_operators {
            return true;
        }

        false
    }
}

impl<'a, Solution, Callbacks> VariableNeighborhoodSearch<'a, Solution, Callbacks> {
    pub fn with_operators<T: IntoIterator<Item = Box<dyn Operator<'a, Solution> + 'a>>>(
        operators: T,
    ) -> Self {
        Self {
            operators: operators.into_iter().collect(),
            callbacks: None,
        }
    }

    pub fn with_callbacks<T: IntoIterator<Item = Box<dyn Operator<'a, Solution> + 'a>>>(
        operators: T,
        callbacks: Callbacks,
    ) -> Self {
        Self {
            operators: operators.into_iter().collect(),
            callbacks: Some(RefCell::new(callbacks)),
        }
    }
}

impl<'a, Solution, Callbacks> LocalSearchHeuristic<Solution>
    for VariableNeighborhoodSearch<'a, Solution, Callbacks>
where
    Solution: Evaluate + Clone,
    Callbacks: VNSCallbacks<Solution>,
{
    fn optimize(&self, initial_solution: Solution) -> Solution {
        // init
        let mut best_solution = initial_solution.clone();
        let mut index_operator = 0;
        let mut iterations_no_improvement = 0;
        loop {
            let ref operator = self.operators[index_operator];

            // explore entire neighborhood
            let mut is_improved_inside_neighborhood = false;
            for neighbor in operator
                .construct_neighborhood(best_solution.clone())
                .into_iter()
            {
                if let Some(callbacks) = &self.callbacks {
                    callbacks.borrow_mut().propose_candidate(&neighbor);
                }
                let fitness = neighbor.evaluate();
                if fitness > best_solution.evaluate() {
                    is_improved_inside_neighborhood = true;
                    best_solution = neighbor;
                }
            }

            if is_improved_inside_neighborhood {
                // improved, so go to first neighborhood
                index_operator = 0;
            } else {
                // go to next neighborhoood
                index_operator += 1;
                iterations_no_improvement += 1;
            }

            if let Some(callbacks) = &self.callbacks {
                if callbacks.borrow_mut().should_terminate() {
                    break;
                }
            }

            if iterations_no_improvement >= self.operators.len() {
                break;
            }
        }

        best_solution
    }
}
