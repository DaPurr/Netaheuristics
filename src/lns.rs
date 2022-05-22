use std::cell::RefCell;

use crate::{termination::TerminationCriteria, Evaluate, ImprovingHeuristic, OperatorSelector};

pub struct LargeNeighborhoodSearch<Solution> {
    destroyers: Vec<Box<dyn Destroyer<Solution = Solution>>>,
    repairers: Vec<Box<dyn Repairer<Solution = Solution>>>,
    selector_destroyer: Box<dyn OperatorSelector>,
    selector_repairer: Box<dyn OperatorSelector>,
    terminator: Box<dyn TerminationCriteria<Solution>>,
    rng: RefCell<Box<dyn rand::RngCore>>,
}

pub trait Destroyer {
    type Solution;
    fn destroy(&self, solution: Self::Solution, rng: &mut dyn rand::RngCore) -> Self::Solution;
}

pub trait Repairer {
    type Solution;
    fn repair(&self, solution: Self::Solution, rng: &mut dyn rand::RngCore) -> Self::Solution;
}

pub struct LNSBuilder<Solution> {
    destroyers: Vec<Box<dyn Destroyer<Solution = Solution>>>,
    repairers: Vec<Box<dyn Repairer<Solution = Solution>>>,
    terminator: Option<Box<dyn TerminationCriteria<Solution>>>,
    selector_destroyer: Option<Box<dyn OperatorSelector>>,
    selector_repairer: Option<Box<dyn OperatorSelector>>,
    rng: Option<Box<dyn rand::RngCore>>,
}

impl<Solution> LargeNeighborhoodSearch<Solution> {
    pub fn builder() -> LNSBuilder<Solution> {
        LNSBuilder {
            destroyers: vec![],
            repairers: vec![],
            terminator: None,
            selector_destroyer: None,
            selector_repairer: None,
            rng: None,
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
            rng: RefCell::new(self.rng.expect("No RNG source specified")),
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

    pub fn rng<T: rand::RngCore + 'static>(mut self, rng: T) -> Self {
        self.rng = Some(Box::new(rng));
        self
    }
}

// impl<Solution> Heuristic<Solution> for LargeNeighborhoodSearch<Solution> {
//     fn optimize(self, solution: Solution) -> Solution
//     where
//         Solution: Clone + Evaluate,
//     {
//         let mut best_solution = solution.clone();
//         let mut incumbent = solution.clone();
//         let mut destroyer_index = self.selector_destroyer.select(&incumbent);
//         let mut repairer_index = self.selector_repairer.select(&incumbent);
//         loop {
//             let destroyer = self.destroyers[destroyer_index].as_ref();
//             let repairer = self.repairers[repairer_index].as_ref();

//             let destroyed = destroyer.destroy(incumbent.clone(), self.rng.borrow_mut().as_mut());
//             let repaired = repairer.repair(destroyed, self.rng.borrow_mut().as_mut());

//             let objective_repaired = repaired.evaluate();
//             if objective_repaired < best_solution.evaluate() {
//                 incumbent = repaired.clone();
//                 best_solution = repaired;
//             } else if objective_repaired < incumbent.evaluate() {
//                 incumbent = repaired;
//             }

//             if self.terminator.terminate(&incumbent) {
//                 break;
//             }

//             destroyer_index = self.selector_destroyer.select(&incumbent);
//             repairer_index = self.selector_repairer.select(&incumbent);
//         }

//         best_solution
//     }
// }

impl<Solution> ImprovingHeuristic<Solution> for LargeNeighborhoodSearch<Solution> {
    fn accept_candidate(&self, candidate: &Solution, incumbent: &Solution) -> bool
    where
        Solution: Evaluate,
    {
        if candidate.evaluate() < incumbent.evaluate() {
            true
        } else {
            false
        }
    }

    fn propose_candidate(&self, incumbent: Solution) -> Solution
    where
        Solution: Evaluate,
    {
        let destroyer_index = self.selector_destroyer.select(&incumbent);
        let repairer_index = self.selector_repairer.select(&incumbent);
        let destroyer = self.destroyers[destroyer_index].as_ref();
        let repairer = self.repairers[repairer_index].as_ref();

        let destroyed = destroyer.destroy(incumbent, self.rng.borrow_mut().as_mut());
        let repaired = repairer.repair(destroyed, self.rng.borrow_mut().as_mut());

        repaired
    }

    fn should_terminate(&self, incumbent: &Solution) -> bool {
        self.terminator.terminate(&incumbent)
    }
}

// todo: fix randomness bug in LNS(?) TSP example shows different objective values, although seed is constant
