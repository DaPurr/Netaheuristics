use rand::{Rng, SeedableRng};

use crate::{
    sa::SimulatedAnnealing, termination::Terminator, vns::VariableNeighborhoodSearch, Evaluate,
    ImprovingHeuristic, Operator, RandomSelector, SequentialSelector, StochasticOperator,
};

#[test]
fn vns_single_operator1() {
    let numbers = vec![9., 8., 7., 8., 9., 7., 5., 0.];
    let n_iterations_max = 10;

    let vns = VariableNeighborhoodSearch::builder()
        .operator(NeighborsUpUntilN::new(&numbers, 1))
        .selector(SequentialSelector::new(1))
        .terminator(Terminator::builder().iterations(n_iterations_max).build())
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
    let n_iterations_max = 10;

    let vns = VariableNeighborhoodSearch::builder()
        .selector(SequentialSelector::new(1))
        .operator(NeighborsUpUntilN::new(&numbers, 3))
        .terminator(Terminator::builder().iterations(n_iterations_max).build())
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
    let n_iterations_max = 10;

    let vns = VariableNeighborhoodSearch::builder()
        .operator(NeighborsUpUntilN::new(&numbers, 1))
        .operator(NeighborsUpUntilN::new(&numbers, 3))
        .selector(SequentialSelector::new(2))
        .terminator(Terminator::builder().iterations(n_iterations_max).build())
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
    let n_iterations_max = 10;

    let vns = VariableNeighborhoodSearch::builder()
        .operator(NeighborsUpUntilN::new(&numbers, 1))
        .operator(NeighborsUpUntilN::new(&numbers, 4))
        .selector(SequentialSelector::new(2))
        .terminator(Terminator::builder().iterations(n_iterations_max).build())
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
    let n_iterations_max = 100;

    let sa = SimulatedAnnealing::builder()
        .operator(NeighborSwap::new(&numbers))
        .selector(RandomSelector::new(rng.clone(), 1))
        .terminator(Terminator::builder().iterations(n_iterations_max).build())
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

impl StochasticOperator for NeighborSwap {
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
