use assert_approx_eq::assert_approx_eq;
use rand::{Rng, SeedableRng};

use crate::{
    sa::SimulatedAnnealing,
    termination::{IterationTerminator, Terminator},
    vns::VariableNeighborhoodSearch,
    Evaluate, ImprovingHeuristic, Operator, ProposalEvaluation, RandomSelector, SelectorAdaptive,
    SequentialSelector,
};

#[test]
fn vns_single_operator1() {
    let numbers = vec![9., 8., 7., 8., 9., 7., 5., 0.];
    let iterations_max = 10;

    let operator: Box<dyn Operator<Solution = Number>> =
        Box::new(NeighborsUpUntilN::new(&numbers, 1));
    let operators = vec![operator];
    let vns = VariableNeighborhoodSearch::builder()
        .selector(SequentialSelector::new(operators))
        .terminator(IterationTerminator::new(iterations_max))
        .build();

    let initial_solution = Number {
        index: 0,
        value: numbers[0],
    };
    let vns_solution = vns.optimize(initial_solution);

    assert_eq!(vns_solution.index, 2)
}

#[test]
fn vns_single_operator2() {
    let numbers = vec![9., 8., 7., 8., 9., 7., 5., 0.];
    let iterations_max = 10;

    let operator: Box<dyn Operator<Solution = Number>> =
        Box::new(NeighborsUpUntilN::new(&numbers, 3));
    let operators = vec![operator];
    let vns = VariableNeighborhoodSearch::builder()
        .selector(SequentialSelector::new(operators))
        .terminator(IterationTerminator::new(iterations_max))
        .build();

    let initial_solution = Number {
        index: 0,
        value: numbers[0],
    };
    let vns_solution = vns.optimize(initial_solution);

    assert_eq!(vns_solution.index, 6)
}

#[test]
fn vns_multiple_operators1() {
    let numbers = vec![9., 8., 7., 8., 9., 7., 5., 0.];
    let iterations_max = 10;

    let operator1: Box<dyn Operator<Solution = Number>> =
        Box::new(NeighborsUpUntilN::new(&numbers, 1));
    let operator2: Box<dyn Operator<Solution = Number>> =
        Box::new(NeighborsUpUntilN::new(&numbers, 3));
    let operators = vec![operator1, operator2];
    let vns = VariableNeighborhoodSearch::builder()
        .selector(SequentialSelector::new(operators))
        .terminator(IterationTerminator::new(iterations_max))
        .build();

    let initial_solution = Number {
        index: 0,
        value: numbers[0],
    };
    let vns_solution = vns.optimize(initial_solution);

    assert_eq!(vns_solution.index, 2)
}

#[test]
fn vns_multiple_operators2() {
    let numbers = vec![9., 8., 7., 8., 9., 7., 5., 0.];
    let iterations_max = 10;

    let operator1: Box<dyn Operator<Solution = Number>> =
        Box::new(NeighborsUpUntilN::new(&numbers, 1));
    let operator2: Box<dyn Operator<Solution = Number>> =
        Box::new(NeighborsUpUntilN::new(&numbers, 4));
    let operators = vec![operator1, operator2];
    let vns = VariableNeighborhoodSearch::builder()
        .selector(SequentialSelector::new(operators))
        .terminator(IterationTerminator::new(iterations_max))
        .build();

    let initial_solution = Number {
        index: 0,
        value: numbers[0],
    };
    let vns_solution = vns.optimize(initial_solution);

    assert_eq!(vns_solution.index, 7)
}

#[test]
fn sa_single_operator() {
    let numbers = vec![9., 8., 7., 8., 9., 7., 5., 0.];
    let rng = rand::rngs::StdRng::seed_from_u64(0);
    let temperature = 100.;
    let iterations_max = 100;

    let operator: Box<dyn Operator<Solution = Number>> = Box::new(NeighborSwap::new(&numbers));
    let sa = SimulatedAnnealing::builder()
        .selector(RandomSelector::new(vec![operator], rng.clone()))
        .terminator(Terminator::builder().iterations(iterations_max).build())
        .rng(rng)
        .temperature(temperature)
        .build();

    let initial_solution = Number {
        index: 0,
        value: numbers[0],
    };

    let sa_solution = sa.optimize(initial_solution);
    assert_eq!(sa_solution.index, 7);
}

#[test]
fn adaptivity_core() {
    let rng = rand::rngs::StdRng::seed_from_u64(0);
    let mut selector = SelectorAdaptive::default_weights(vec![1, 2, 3], 1., rng);
    assert_approx_eq!(selector.weights[0], 1.);
    assert_approx_eq!(selector.weights[1], 1.);
    assert_approx_eq!(selector.weights[2], 1.);

    selector.index_last_selection.replace(Some(0));
    selector.feedback(ProposalEvaluation::ImprovedBest);
    assert_approx_eq!(selector.weights[0], 4.);
    assert_approx_eq!(selector.weights[1], 1.);
    assert_approx_eq!(selector.weights[2], 1.);

    selector.index_last_selection.replace(Some(2));
    selector.feedback(ProposalEvaluation::Accept);
    assert_approx_eq!(selector.weights[0], 4.);
    assert_approx_eq!(selector.weights[1], 1.);
    assert_approx_eq!(selector.weights[2], 2.);
}

#[derive(Clone, Debug)]
struct Number {
    value: f32,
    index: usize,
}

struct NeighborsUpUntilN {
    numbers: Vec<Number>,
    index_cursor: Option<usize>,
    iter: isize,
    n: usize,
}

struct NeighborSwap {
    numbers: Vec<Number>,
}

impl NeighborSwap {
    pub fn new(numbers: &[f32]) -> Self {
        Self {
            numbers: numbers
                .iter()
                .enumerate()
                .map(|(index, x)| Number { index, value: *x })
                .collect(),
        }
    }
}

impl Operator for NeighborSwap {
    type Solution = Number;
    fn shake(&self, solution: Number, rng: &mut dyn rand::RngCore) -> Self::Solution {
        let index = solution.index;
        let mut options = vec![];
        if index as isize - 1 >= 0 {
            options.push(self.numbers[index - 1].clone());
        }
        if index + 1 < self.numbers.len() {
            options.push(self.numbers[index + 1].clone());
        }

        if options.len() == 1 {
            options.remove(0)
        } else {
            options.remove(rng.gen_range(0..2))
        }
    }
}

impl NeighborsUpUntilN {
    fn new(numbers: &Vec<f32>, n: usize) -> Self {
        Self {
            index_cursor: None,
            n,
            iter: 0,
            numbers: numbers
                .iter()
                .enumerate()
                .map(|(index, value)| Number {
                    index,
                    value: *value,
                })
                .collect(),
        }
    }
}

impl Evaluate for Number {
    fn evaluate(&self) -> f32 {
        self.value
    }
}

impl Operator for NeighborsUpUntilN {
    type Solution = Number;
    fn construct_neighborhood(&self, solution: Number) -> Box<dyn Iterator<Item = Number>> {
        let index_cursor = solution.index;
        Box::new(Self {
            index_cursor: Some(index_cursor),
            iter: 0,
            n: self.n,
            numbers: self.numbers.clone(),
        })
    }
}

impl Iterator for NeighborsUpUntilN {
    type Item = Number;

    fn next(&mut self) -> Option<Self::Item> {
        match self.index_cursor {
            Some(index_cursor) => {
                if self.iter > 1 {
                    return None;
                }

                let lb = 0 as isize;
                let ub = self.numbers.len() as isize - 1;
                let n = self.n as isize;
                let iter = self.iter as isize;

                let index = index_cursor as isize - n * (1 - 2 * iter);
                if index < lb {
                    self.iter += 1;
                    return self.next();
                } else if index <= ub {
                    let item = self.numbers[index as usize].clone();
                    self.iter += 1;
                    Some(item)
                } else {
                    None
                }
            }
            None => None,
        }
    }
}
