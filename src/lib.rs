pub trait Evaluate {
    fn evaluate(&self) -> f32;
}

pub trait Neighborhood<Solution> {
    fn construct(&self, solution: &Solution) -> Box<dyn Iterator<Item = Solution>>;
}

pub trait LocalSearchHeuristic<Solution> {
    fn optimize(&self, solution: Solution) -> Option<Solution>;
}

pub struct VariableNeighborhoodSearch<Solution> {
    neighborhoods: Vec<Box<dyn Neighborhood<Solution>>>,
}

impl<Solution> VariableNeighborhoodSearch<Solution> {
    pub fn new<T: IntoIterator<Item = Box<dyn Neighborhood<Solution>>>>(neighborhoods: T) -> Self {
        Self {
            neighborhoods: neighborhoods.into_iter().collect(),
        }
    }
}

impl<Solution> LocalSearchHeuristic<Solution> for VariableNeighborhoodSearch<Solution>
where
    Solution: Evaluate,
{
    fn optimize(&self, solution: Solution) -> Option<Solution> {
        // init
        let mut best_solution = None;
        let mut index_neighborhood = 0;

        loop {
            let ref neighborhood = self.neighborhoods[index_neighborhood];

            // explore entire neighborhood
            let mut is_improved_inside_neighborhood = false;
            for neighbor in neighborhood.construct(&solution).into_iter() {
                let fitness = neighbor.evaluate();

                match &best_solution {
                    None => best_solution = Some(neighbor),
                    Some(x) => {
                        if fitness > x.evaluate() {
                            is_improved_inside_neighborhood = true;
                            best_solution = Some(neighbor);
                        }
                    }
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
                if index_neighborhood >= self.neighborhoods.len()
                    && !is_improved_inside_neighborhood
                {
                    break;
                }
            }
        }

        best_solution
    }
}
