use heuristics::{Evaluate, LocalSearchHeuristic, Operator, VariableNeighborhoodSearch};

fn main() {
    let numbers = vec![0., 1., 2., 1., 0., 2., 4., 9.];

    let neighbor1: Box<dyn Operator<Number>> = Box::new(NeighborsUpUntilN::new(&numbers, 1));
    let neighbor2: Box<dyn Operator<Number>> = Box::new(NeighborsUpUntilN::new(&numbers, 4));
    let vns = VariableNeighborhoodSearch::new([neighbor1, neighbor2]);

    let initial_solution = Number {
        index: 0,
        value: numbers[0],
    };
    let vns_solution = vns.optimize(initial_solution);

    println!(
        "numbers: {}",
        numbers
            .into_iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );
    println!("vns solution: {}", vns_solution.value);
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

impl<'a> Operator<'a, Number> for NeighborsUpUntilN {
    fn construct_neighborhood(&self, solution: Number) -> Box<dyn Iterator<Item = Number> + 'a> {
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

                let index = index_cursor as isize - self.n as isize + 2 * self.iter;
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
