use crate::{termination::TerminationCriteria, Evaluate, Heuristic, OperatorSelector};

pub struct LargeNeighborhoodSearch<Solution> {
    destroyers: Vec<Box<dyn Destroyer<Solution = Solution>>>,
    repairers: Vec<Box<dyn Repairer<Solution = Solution>>>,
    selector_destroyer: Box<dyn OperatorSelector>,
    selector_repairer: Box<dyn OperatorSelector>,
    terminator: Box<dyn TerminationCriteria<Solution>>,
}

pub trait Destroyer {
    type Solution;
    fn destroy(&self, solution: Self::Solution) -> Self::Solution;
}

pub trait Repairer {
    type Solution;
    fn repair(&self, solution: Self::Solution) -> Self::Solution;
}

pub struct LNSBuilder<Solution> {
    destroyers: Vec<Box<dyn Destroyer<Solution = Solution>>>,
    repairers: Vec<Box<dyn Repairer<Solution = Solution>>>,
    terminator: Option<Box<dyn TerminationCriteria<Solution>>>,
    selector_destroyer: Option<Box<dyn OperatorSelector>>,
    selector_repairer: Option<Box<dyn OperatorSelector>>,
}

impl<Solution> LargeNeighborhoodSearch<Solution> {
    pub fn builder() -> LNSBuilder<Solution> {
        LNSBuilder {
            destroyers: vec![],
            repairers: vec![],
            terminator: None,
            selector_destroyer: None,
            selector_repairer: None,
        }
    }
}

impl<Solution> LNSBuilder<Solution> {
    pub fn build(self) -> LargeNeighborhoodSearch<Solution> {
        LargeNeighborhoodSearch {
            destroyers: self.destroyers,
            repairers: self.repairers,
            selector_destroyer: self
                .selector_destroyer
                .expect("No destroyer selector specified"),
            selector_repairer: self
                .selector_repairer
                .expect("No repairer selector specified"),
            terminator: self.terminator.expect("No termination criteria specified"),
        }
    }

    pub fn terminator(mut self, terminator: Box<dyn TerminationCriteria<Solution>>) -> Self {
        self.terminator = Some(terminator);
        self
    }

    pub fn destroyer<T: Destroyer<Solution = Solution> + 'static>(mut self, destroyer: T) -> Self {
        self.destroyers.push(Box::new(destroyer));
        self
    }

    pub fn repairer<T: Repairer<Solution = Solution> + 'static>(mut self, repairer: T) -> Self {
        self.repairers.push(Box::new(repairer));
        self
    }

    pub fn selector_destroyer<T: OperatorSelector + 'static>(mut self, selector: T) -> Self {
        self.selector_destroyer = Some(Box::new(selector));
        self
    }

    pub fn selector_repairer<T: OperatorSelector + 'static>(mut self, repairer: T) -> Self {
        self.selector_repairer = Some(Box::new(repairer));
        self
    }
}

impl<Solution: Clone + Evaluate> Heuristic for LargeNeighborhoodSearch<Solution> {
    type Solution = Solution;
    fn optimize(self, solution: Solution) -> Solution {
        let mut best_solution = solution.clone();
        let mut incumbent = solution.clone();
        let mut destroyer_index = self.selector_destroyer.select(&incumbent);
        let mut repairer_index = self.selector_repairer.select(&incumbent);
        loop {
            let destroyer = self.destroyers[destroyer_index].as_ref();
            let repairer = self.repairers[repairer_index].as_ref();

            let destroyed = destroyer.destroy(incumbent.clone());
            let repaired = repairer.repair(destroyed);

            if repaired.evaluate() < best_solution.evaluate() {
                incumbent = repaired.clone();
                best_solution = repaired;
            } else if repaired.evaluate() < incumbent.evaluate() {
                incumbent = repaired;
            }

            if self.terminator.terminate(&incumbent) {
                break;
            }

            destroyer_index = self.selector_destroyer.select(&incumbent);
            repairer_index = self.selector_repairer.select(&incumbent);
        }

        best_solution
    }
}
