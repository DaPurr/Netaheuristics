use rand::Rng;

use crate::{Evaluate, Operator};

#[derive(Clone, Debug)]
pub(crate) struct Number {
    value: f32,
    index: usize,
}

pub(crate) struct NeighborsUpUntilN {
    numbers: Vec<Number>,
    index_cursor: Option<usize>,
    iter: isize,
    n: usize,
}

pub(crate) struct NeighborSwap {
    numbers: Vec<Number>,
}

impl Number {
    pub fn new(index: usize, value: f32) -> Self {
        Self { value, index }
    }
    pub fn index(&self) -> usize {
        self.index
    }
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
    pub(crate) fn new(numbers: &Vec<f32>, n: usize) -> Self {
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
