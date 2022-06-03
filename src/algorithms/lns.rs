//! _large neighborhood search_
use std::cell::RefCell;

use crate::{
    selectors::OperatorSelector, termination::TerminationCriteria, Evaluate, ImprovingHeuristic,
};

/// Large Neighborhood Search implementation.
pub struct LargeNeighborhoodSearch<Solution> {
    selector_destroyer: Box<dyn OperatorSelector<Solution>>,
    selector_repairer: Box<dyn OperatorSelector<Solution>>,
    terminator: Box<dyn TerminationCriteria<Solution>>,
    rng: RefCell<Box<dyn rand::RngCore>>,
}

/// Builder design pattern for [LargeNeighborhoodSearch].
pub struct LNSBuilder<Solution> {
    terminator: Option<Box<dyn TerminationCriteria<Solution>>>,
    selector_destroyer: Option<Box<dyn OperatorSelector<Solution>>>,
    selector_repairer: Option<Box<dyn OperatorSelector<Solution>>>,
    rng: Option<Box<dyn rand::RngCore>>,
}

impl<Solution> LargeNeighborhoodSearch<Solution> {
    pub fn builder() -> LNSBuilder<Solution> {
        LNSBuilder {
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

    pub fn selector_destroyer<T: OperatorSelector<Solution> + 'static>(
        mut self,
        selector: T,
    ) -> Self {
        self.selector_destroyer = Some(Box::new(selector));
        self
    }

    pub fn selector_repairer<T: OperatorSelector<Solution> + 'static>(
        mut self,
        repairer: T,
    ) -> Self {
        self.selector_repairer = Some(Box::new(repairer));
        self
    }

    pub fn rng<T: rand::RngCore + 'static>(mut self, rng: T) -> Self {
        self.rng = Some(Box::new(rng));
        self
    }
}

impl<Solution> ImprovingHeuristic<Solution> for LargeNeighborhoodSearch<Solution> {
    /// Accept a candidate iff it is an improvement.
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

    /// Select a destroy and repair method, then return the destroyed and repaired ```incumbent```.
    fn propose_candidate(&self, incumbent: Solution) -> Solution
    where
        Solution: Evaluate,
    {
        let destroyer = self.selector_destroyer.select(&incumbent);
        let repairer = self.selector_repairer.select(&incumbent);

        let destroyed = destroyer.shake(incumbent, self.rng.borrow_mut().as_mut());
        let repaired = repairer.shake(destroyed, self.rng.borrow_mut().as_mut());

        repaired
    }

    /// Terminate iff the termination criteria are satisfied.
    fn should_terminate(&self, incumbent: &Solution) -> bool {
        self.terminator.terminate(&incumbent)
    }
}
