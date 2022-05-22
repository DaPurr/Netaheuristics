use rand::Rng;

use crate::{
    termination::TerminationCriteria, Evaluate, Heuristic, OperatorSelector, StochasticOperator,
};

pub struct SimulatedAnnealing<Solution> {
    operators: Vec<Box<dyn StochasticOperator<Solution = Solution>>>,
    selector: Box<dyn OperatorSelector>,
    terminator: Box<dyn TerminationCriteria<Solution>>,
    rng: Box<dyn rand::RngCore>,
    temperature: f32,
}

pub struct SABuilder<Solution> {
    selector: Option<Box<dyn OperatorSelector>>,
    terminator: Option<Box<dyn TerminationCriteria<Solution>>>,
    operators: Vec<Box<dyn StochasticOperator<Solution = Solution>>>,
    rng: Option<Box<dyn rand::RngCore>>,
    temperature: Option<f32>,
}

impl<Solution> SimulatedAnnealing<Solution> {
    pub fn builder() -> SABuilder<Solution> {
        SABuilder {
            operators: vec![],
            selector: None,
            terminator: None,
            temperature: None,
            rng: None,
        }
    }
}

impl<Solution> SABuilder<Solution> {
    pub fn build(self) -> SimulatedAnnealing<Solution> {
        SimulatedAnnealing {
            operators: self.operators,
            rng: self.rng.expect("No RNG source specified"),
            selector: self
                .selector
                .expect("No operator selection strategy specified"),
            terminator: self.terminator.expect("No termination criteria specified"),
            temperature: self.temperature.expect("No initial temperature specified"),
        }
    }

    pub fn terminator(mut self, criterium: Box<dyn TerminationCriteria<Solution>>) -> Self {
        self.terminator = Some(criterium);
        self
    }

    pub fn operator<T: StochasticOperator<Solution = Solution> + 'static>(
        mut self,
        operator: T,
    ) -> Self {
        self.operators.push(Box::new(operator));
        self
    }

    pub fn selector<T: OperatorSelector + 'static>(mut self, selector: T) -> Self {
        self.selector = Some(Box::new(selector));
        self
    }

    pub fn rng<T: rand::RngCore + 'static>(mut self, rng: T) -> Self {
        self.rng = Some(Box::new(rng));
        self
    }

    pub fn temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
}

impl<Solution: Evaluate + Clone> Heuristic for SimulatedAnnealing<Solution> {
    type Solution = Solution;
    fn optimize(mut self, solution: Self::Solution) -> Self::Solution {
        // init
        let mut incumbent = solution;
        let mut best_solution = incumbent.clone();
        let mut operator_index = self.selector.select(&incumbent);
        let mut operator = self.operators[operator_index].as_ref();

        loop {
            let r: f32 = self.rng.gen();

            let candidate = operator.shake(incumbent.clone(), &mut self.rng);
            if candidate.evaluate() < best_solution.evaluate() {
                incumbent = candidate.clone();
                best_solution = candidate.clone();
            } else if r
                <= compute_probability(self.temperature, incumbent.evaluate(), candidate.evaluate())
            {
                incumbent = candidate.clone();
            }

            operator_index = self.selector.select(&incumbent);
            operator = self.operators[operator_index].as_ref();

            if self.terminator.terminate(&candidate) {
                break;
            }
        }

        best_solution
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
