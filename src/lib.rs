#[cfg(test)]
mod test;
pub mod vns;

pub trait Evaluate {
    fn evaluate(&self) -> f32;
}

pub trait Operator<Solution> {
    fn construct_neighborhood(&self, solution: Solution) -> Box<dyn Iterator<Item = Solution>>;

    #[allow(unused_variables)]
    fn shake(&self, solution: Solution, rng: &mut dyn rand::RngCore) -> Solution {
        solution
    }

    fn find_best_neighbor(&self, solution: Solution) -> Solution
    where
        Solution: Evaluate,
    {
        let mut winner: Option<Solution> = None;
        for neighbor in self.construct_neighborhood(solution) {
            if let Some(x) = &winner {
                if neighbor.evaluate() > x.evaluate() {
                    winner = Some(neighbor);
                }
            } else {
                winner = Some(neighbor);
            }
        }
        winner.expect("neighborhood was empty")
    }
}

pub trait LocalSearchHeuristic {
    type Solution: Evaluate + Clone;
    fn optimize(self, solution: Self::Solution) -> Self::Solution;
}

pub trait TerminationCriteria<State> {
    fn terminate(&self, state: &State) -> bool;
}
