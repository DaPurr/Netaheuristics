use crate::{Evaluate, LocalSearchHeuristic, Operator};

pub struct VariableNeighborhoodSearch<'a, Solution> {
    operators: Vec<Box<dyn Operator<'a, Solution> + 'a>>,
}

impl<'a, Solution> VariableNeighborhoodSearch<'a, Solution> {
    pub fn new<T: IntoIterator<Item = Box<dyn Operator<'a, Solution> + 'a>>>(operators: T) -> Self {
        Self {
            operators: operators.into_iter().collect(),
        }
    }
}

impl<'a, Solution> LocalSearchHeuristic<Solution> for VariableNeighborhoodSearch<'a, Solution>
where
    Solution: Evaluate + Clone,
{
    fn optimize(&self, initial_solution: Solution) -> Solution {
        // init
        let mut best_solution = initial_solution.clone();
        let mut index_operator = 0;
        loop {
            let ref operator = self.operators[index_operator];

            // explore entire neighborhood
            let mut is_improved_inside_neighborhood = false;
            for neighbor in operator
                .construct_neighborhood(best_solution.clone())
                .into_iter()
            {
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

                // check termination criteria
                // did we reach the end without improvement?
                if index_operator >= self.operators.len() && !is_improved_inside_neighborhood {
                    break;
                }
            }
        }

        best_solution
    }
}
