//! _simulated annealing_.
use std::{cell::RefCell, ops::MulAssign};

use crate::{
    selectors::OperatorSelector, termination::TerminationCriteria, Evaluate, ImprovingHeuristic,
    Operator,
};

use rand::Rng;

/// Simulated Annealing implementation.
pub struct SimulatedAnnealing<Solution> {
    selector: Box<dyn OperatorSelector<Solution>>,
    terminator: Box<dyn TerminationCriteria<Solution>>,
    rng: RefCell<Box<dyn rand::RngCore>>,
    cooling_schedule: Box<dyn CoolingSchedule>,
}

/// Builder design pattern for [SimulatedAnnealing].
pub struct SABuilder<Solution> {
    selector: Option<Box<dyn OperatorSelector<Solution>>>,
    terminator: Option<Box<dyn TerminationCriteria<Solution>>>,
    operators: Vec<Box<dyn Operator<Solution = Solution>>>,
    rng: Option<Box<dyn rand::RngCore>>,
    cooling_schedule: Option<Box<dyn CoolingSchedule>>,
}

/// Cool the system according to a schedule
pub trait CoolingSchedule {
    fn cool(&self);
    fn temperature(&self) -> f32;
}

/// Cool, every iteration, using a constant factor
pub struct FactorSchedule {
    temperature: RefCell<f32>,
    cooling_factor: f32,
}

impl FactorSchedule {
    pub fn new(initial_temperature: f32, decay: f32) -> Self {
        Self {
            temperature: RefCell::new(initial_temperature),
            cooling_factor: decay,
        }
    }
}

impl CoolingSchedule for FactorSchedule {
    fn cool(&self) {
        self.temperature
            .borrow_mut()
            .mul_assign(1. - self.cooling_factor)
    }

    fn temperature(&self) -> f32 {
        *self.temperature.borrow()
    }
}

impl<Solution> SimulatedAnnealing<Solution> {
    pub fn builder() -> SABuilder<Solution> {
        SABuilder {
            operators: vec![],
            selector: None,
            terminator: None,
            rng: None,
            cooling_schedule: None,
        }
    }
}

impl<Solution> SABuilder<Solution> {
    /// Build the configured Simulated Annealing heuristic
    pub fn build(self) -> SimulatedAnnealing<Solution> {
        SimulatedAnnealing {
            rng: RefCell::new(self.rng.expect("No RNG source specified")),
            selector: self
                .selector
                .expect("No operator selection strategy specified"),
            terminator: self.terminator.expect("No termination criteria specified"),
            cooling_schedule: self
                .cooling_schedule
                .expect("No cooling schedule specified"),
        }
    }

    /// Set termination criteria
    pub fn terminator(mut self, criterium: Box<dyn TerminationCriteria<Solution>>) -> Self {
        self.terminator = Some(criterium);
        self
    }

    /// Add an operator
    pub fn operator<T: Operator<Solution = Solution> + 'static>(mut self, operator: T) -> Self {
        self.operators.push(Box::new(operator));
        self
    }

    /// Set operator selector
    pub fn selector<T: OperatorSelector<Solution> + 'static>(mut self, selector: T) -> Self {
        self.selector = Some(Box::new(selector));
        self
    }

    /// Set source of randomness
    pub fn rng<T: rand::RngCore + 'static>(mut self, rng: T) -> Self {
        self.rng = Some(Box::new(rng));
        self
    }

    /// Set initial temperature
    pub fn cooling_schedule<T: CoolingSchedule + 'static>(mut self, cooling_schedule: T) -> Self {
        self.cooling_schedule = Some(Box::new(cooling_schedule));
        self
    }
}

impl<Solution> ImprovingHeuristic<Solution> for SimulatedAnnealing<Solution> {
    /// Accept iff the ```candidate``` is better than the ```incumbent```, or otherwise with a probabilty equal to the acceptance probability.
    ///
    /// The acceptance probability is calculated as exp(-delta / Temperature).
    fn accept_candidate(&self, candidate: &Solution, incumbent: &Solution) -> bool
    where
        Solution: Evaluate,
    {
        let temperature = self.cooling_schedule.temperature();
        let r: f32 = self.rng.borrow_mut().gen();
        if candidate.evaluate() < incumbent.evaluate() {
            true
        } else if r <= compute_probability(temperature, incumbent.evaluate(), candidate.evaluate())
        {
            true
        } else {
            false
        }
    }

    /// Select an operator and draw a random neighbor.
    fn propose_candidate(&self, incumbent: Solution) -> Solution
    where
        Solution: Evaluate,
    {
        let operator = self.selector.select(&incumbent);
        let candidate = operator.shake(incumbent, self.rng.borrow_mut().as_mut());
        self.cooling_schedule.cool();
        candidate
    }

    /// Test whether the termination criteria are fulfilled.
    fn should_terminate(&self, incumbent: &Solution) -> bool {
        self.terminator.terminate(&incumbent)
    }
}

fn compute_probability(
    temperature: f32,
    objective_incumbent: f32,
    objective_candidate: f32,
) -> f32 {
    let delta = objective_incumbent - objective_candidate;
    if delta < 0. {
        (-delta / temperature).exp()
    } else {
        1.
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use crate::{
        algorithms::sa::{FactorSchedule, SimulatedAnnealing},
        selectors::RandomSelector,
        termination::Terminator,
        test::{NeighborSwap, Number},
        ImprovingHeuristic,
    };

    #[test]
    fn sa_single_operator() {
        let numbers = vec![9., 8., 7., 8., 9., 7., 5., 0.];
        let rng = rand::rngs::StdRng::seed_from_u64(0);
        let temperature = 100.;
        let iterations_max = 100;
        let schedule = FactorSchedule::new(temperature, 0.05);

        let operator = NeighborSwap::new(&numbers);
        let sa = SimulatedAnnealing::builder()
            .selector(RandomSelector::new(rng.clone()).option(operator))
            .terminator(Terminator::builder().iterations(iterations_max).build())
            .rng(rng)
            .cooling_schedule(schedule)
            .build();

        let initial_solution = Number::new(0, numbers[0]);
        let sa_solution = sa.optimize(initial_solution);
        assert_eq!(sa_solution.index(), 7);
    }
}
