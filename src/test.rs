use crate::{
    termination::Terminator,
    vns::{SequentialSelector, VariableNeighborhoodSearch},
    Evaluate, LocalSearchHeuristic, Operator,
};

#[test]
fn vns_single_operator() {
    let numbers = vec![0., 1., 2., 1., 0., 2., 4., 9.];
    let n_iterations_max = 10;

    let vns = VariableNeighborhoodSearch::builder()
        .operator(NeighborsUpUntilN::new(&numbers, 1))
        .selector(SequentialSelector::default())
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
    let numbers = vec![0., 1., 2., 1., 0., 2., 4., 9.];
    let n_iterations_max = 10;

    let vns = VariableNeighborhoodSearch::builder()
        .selector(SequentialSelector::default())
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
    let numbers = vec![0., 1., 2., 1., 0., 2., 4., 9.];
    let n_iterations_max = 10;

    let vns = VariableNeighborhoodSearch::builder()
        .operator(NeighborsUpUntilN::new(&numbers, 1))
        .operator(NeighborsUpUntilN::new(&numbers, 3))
        .selector(SequentialSelector::default())
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
    let numbers = vec![0., 1., 2., 1., 0., 2., 4., 9.];
    let n_iterations_max = 10;

    let vns = VariableNeighborhoodSearch::builder()
        .operator(NeighborsUpUntilN::new(&numbers, 1))
        .operator(NeighborsUpUntilN::new(&numbers, 4))
        .selector(SequentialSelector::default())
        .terminator(Terminator::builder().iterations(n_iterations_max).build())
        .build();

    let initial_solution = Number {
        index: 0,
        value: numbers[0],
    };
    let vns_solution = vns.optimize(initial_solution);

    assert_eq!(vns_solution.index, 7)
}

#[derive(Clone)]
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

impl Operator<Number> for NeighborsUpUntilN {
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
