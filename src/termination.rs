//! All types required to model termination criteria.

use std::{
    cell::RefCell,
    ops::AddAssign,
    time::{Duration, SystemTime},
};

/// Models a type representing a heuristic's termination criteria.
pub trait TerminationCriteria<Solution> {
    fn terminate(&self, solution: &Solution) -> bool;
}

/// Terminates when at least one termination criterium evaluates to true.
pub struct OrTerminator<Solution> {
    terminators: Vec<Box<dyn TerminationCriteria<Solution>>>,
}

/// Terminates after ```n``` iterations have been performed.
pub struct IterationTerminator {
    n: usize,
    iteration: RefCell<usize>,
}

/// Terminates after a certain amount of time has passed. This criterium does finish the iteration, however.
pub struct TimeTerminator {
    time_end: SystemTime,
}

/// Terminates when all termination criteria evaluate to true.
pub struct AndTerminator<Solution> {
    terminators: Vec<Box<dyn TerminationCriteria<Solution>>>,
}

enum AggregateTermination {
    Any,
    All,
}

/// Builder design pattern to construct termination criteria.
pub struct TerminatorBuilder<Solution> {
    terminators: Vec<Box<dyn TerminationCriteria<Solution>>>,
    aggregator: AggregateTermination,
}

pub struct Terminator;

impl Terminator {
    /// Construct a builder for termination criteria.
    pub fn builder<Solution>() -> TerminatorBuilder<Solution> {
        TerminatorBuilder {
            aggregator: AggregateTermination::Any,
            terminators: vec![],
        }
    }
}

impl<Solution> TerminatorBuilder<Solution> {
    /// Build termination criteria on the stack.
    pub fn build(self) -> Box<dyn TerminationCriteria<Solution>>
    where
        Solution: 'static,
    {
        match self.aggregator {
            AggregateTermination::All => Box::new(AndTerminator {
                terminators: self.terminators,
            }),
            AggregateTermination::Any => Box::new(OrTerminator {
                terminators: self.terminators,
            }),
        }
    }

    /// Add a termination criterium, to be aggregated later.
    pub fn criterium<T: TerminationCriteria<Solution> + 'static>(mut self, criterium: T) -> Self {
        self.terminators.push(Box::new(criterium));
        self
    }

    /// Add a limit on the number of iterations.
    pub fn iterations(mut self, n: usize) -> Self {
        self.terminators.push(Box::new(IterationTerminator {
            n,
            iteration: RefCell::new(0),
        }));
        self
    }

    /// Add a time limit.
    pub fn time_max(mut self, seconds: u64) -> Self {
        let time_end = std::time::SystemTime::now() + Duration::from_secs(seconds);
        self.terminators.push(Box::new(TimeTerminator { time_end }));
        self
    }

    /// Construct an aggregating termination criterium which only evaluates to true if all criteria do so.
    pub fn all(mut self) -> Self {
        self.aggregator = AggregateTermination::All;
        self
    }

    /// Construct an aggregating termination criterium which only evaluates to true if at least one criterium does so.
    pub fn any(mut self) -> Self {
        self.aggregator = AggregateTermination::Any;
        self
    }
}

impl<Solution> TerminationCriteria<Solution> for OrTerminator<Solution> {
    fn terminate(&self, solution: &Solution) -> bool {
        self.terminators.iter().any(|x| x.terminate(solution))
    }
}

impl<Solution> TerminationCriteria<Solution> for AndTerminator<Solution> {
    fn terminate(&self, solution: &Solution) -> bool {
        self.terminators.iter().all(|x| x.terminate(solution))
    }
}

impl<Solution> TerminationCriteria<Solution> for IterationTerminator {
    fn terminate(&self, _solution: &Solution) -> bool {
        self.iteration.borrow_mut().add_assign(1);
        if *self.iteration.borrow() == self.n {
            true
        } else {
            false
        }
    }
}

impl<Solution> TerminationCriteria<Solution> for TimeTerminator {
    fn terminate(&self, _solution: &Solution) -> bool {
        let now = std::time::SystemTime::now();
        now >= self.time_end
    }
}
