#[cfg(test)]
mod test;
pub mod vns;

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

pub trait TerminationCriteria {
    fn terminate(&self) -> bool;
}
