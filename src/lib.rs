pub trait Evaluate {
    fn evaluate(&self) -> f32;
}

pub trait Operator<'a, Solution> {
    fn construct_neighborhood(&self, solution: Solution)
        -> Box<dyn Iterator<Item = Solution> + 'a>;
}

pub trait LocalSearchHeuristic<Solution> {
    fn optimize(&self, solution: Solution) -> Solution;
}

pub struct VariableNeighborhoodSearch<'a, Solution> {
    operators: Vec<Box<dyn Operator<'a, Solution> + 'a>>,
}

impl<'a, Solution> VariableNeighborhoodSearch<'a, Solution> {
    pub fn new<T: IntoIterator<Item = Box<dyn Operator<'a, Solution> + 'a>>>(
        neighborhoods: T,
    ) -> Self {
        Self {
            operators: neighborhoods.into_iter().collect(),
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
        let mut index_neighborhood = 0;
        loop {
            let ref operator = self.operators[index_neighborhood];

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
                index_neighborhood = 0;
            } else {
                // go to next neighborhoood
                index_neighborhood += 1;

                // check termination criteria
                // did we reach the end without improvement?
                if index_neighborhood >= self.operators.len() && !is_improved_inside_neighborhood {
                    break;
                }
            }
        }

        best_solution
    }
}
