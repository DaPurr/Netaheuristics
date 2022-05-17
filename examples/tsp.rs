use heuristics::{
    vns::{SequentialSelector, TerminationCriteriaDefault, VariableNeighborhoodSearch},
    Evaluate, LocalSearchHeuristic, Operator,
};
use rand::{Rng, RngCore, SeedableRng};

fn main() {
    // init
    let n = 10;
    let width = 100.;
    let height = 100.;
    let seed = 0;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    // create random cities
    let cities: Vec<City> = (0..n)
        .map(|_| create_random_city(width, height, &mut rng))
        .collect();
    let cities = Box::new(cities);

    let random_tour = construct_random_tour(&mut cities.clone().into_iter(), &mut rng);
    let greedy_tour = construct_greedy_tour(&mut cities.clone().into_iter(), &mut rng);

    // optimize with VNS
    let operator_2opt = TwoOpt::new(cities.as_slice());
    let operator_3opt = ThreeOpt::new(cities.as_slice());

    let vns = VariableNeighborhoodSearch::builder()
        .selector(SequentialSelector::default())
        .operator(operator_2opt)
        .operator(operator_3opt)
        .terminator(TerminationCriteriaDefault::new(10))
        .build();
    let vns_tour = vns.optimize(random_tour.clone());

    let length_random_tour = random_tour.evaluate();
    let length_greedy_tour = greedy_tour.evaluate();
    let length_vns_tour = vns_tour.evaluate();

    println!("random tour length: {}", length_random_tour);
    println!("greedy tour length: {}", length_greedy_tour);
    println!("vns tour length: {}", length_vns_tour);
}

#[derive(Clone)]
struct City {
    x: f32,
    y: f32,
}

#[derive(Clone)]
struct Tour {
    cities: Vec<City>,
}

#[derive(Clone)]
struct TwoOpt {
    tour: Option<Tour>,
    cities: Box<Vec<City>>,
    index1: usize,
    index2: usize,
}

struct ThreeOpt {
    tour: Option<Tour>,
    cities: Box<Vec<City>>,
    permutation: ThreeOptPermutation,
    index1: usize,
    index2: usize,
    index3: usize,
}

enum ThreeOptPermutation {
    One,
    Two,
    Three,
    Four,
}

impl<'a> TwoOpt {
    fn new(cities: &[City]) -> Self {
        Self {
            tour: None,
            cities: Box::new(cities.to_owned()),
            index1: 0,
            index2: 0,
        }
    }

    // advance index2 before index1
    fn advance(&mut self) -> bool {
        let n = self.cities.len();
        // index1 free
        if self.index2 + 1 < n {
            self.index2 += 1;
        }
        // index1 blocked, index2 blocked
        else if self.index1 + 1 == n {
            return false;
        }
        // index1 free, index2 blocked
        else if self.index1 + 1 < n {
            self.index2 = 0;
            self.index1 += 1;
        } else {
            panic!("unknown state");
        }

        true
    }
}

impl<'a> Iterator for TwoOpt {
    type Item = Tour;

    fn next(&mut self) -> Option<Self::Item> {
        // advance by one
        if !self.advance() {
            return None;
        }

        // return tour with the two cities swapped
        if let Some(tour) = &self.tour {
            Some(tour.swap(self.index1, self.index2))
        } else {
            None
        }
    }
}

impl Operator<Tour> for TwoOpt {
    fn construct_neighborhood(&self, solution: Tour) -> Box<dyn Iterator<Item = Tour>> {
        let mut neighborhood = Self::new(self.cities.as_ref());
        neighborhood.tour = Some(solution);
        Box::new(neighborhood)
    }
}

impl ThreeOpt {
    fn new(cities: &[City]) -> Self {
        Self {
            tour: None,
            cities: Box::new(cities.to_owned()),
            permutation: ThreeOptPermutation::One,
            index1: 0,
            index2: 0,
            index3: 0,
        }
    }

    // advance index3, index2, index1
    fn advance(&mut self) -> bool {
        match self.permutation {
            ThreeOptPermutation::One => {
                self.permutation = ThreeOptPermutation::Two;

                true
            }
            ThreeOptPermutation::Two => {
                self.permutation = ThreeOptPermutation::Three;

                true
            }
            ThreeOptPermutation::Three => {
                self.permutation = ThreeOptPermutation::Four;

                true
            }
            ThreeOptPermutation::Four => {
                self.permutation = ThreeOptPermutation::One;

                let n = self.cities.len();
                // index3 free
                if self.index3 + 1 < n {
                    self.index3 += 1;
                }
                // index3 blocked, index2 free
                else if self.index2 + 1 < n {
                    self.index2 += 1;
                    self.index3 = 0;
                }
                // index3 blocked, index2 blocked, index1 free
                else if self.index1 + 1 < n {
                    self.index1 += 1;
                    self.index2 = 0;
                    self.index3 = 0;
                }
                // index1, index2, index3 blocked
                else if self.index3 + 1 == n && self.index2 + 1 == n && self.index1 + 1 == n {
                    return false;
                } else {
                    panic!("invalid state")
                }

                true
            }
        }
    }
}

impl Iterator for ThreeOpt {
    type Item = Tour;

    fn next(&mut self) -> Option<Self::Item> {
        // no tour means no neighborhood
        if let None = self.tour {
            return None;
        }

        // advance by one
        if !self.advance() {
            return None;
        }

        // return 3-opt permutation of tour
        match (&self.tour, &self.permutation) {
            (Some(tour), ThreeOptPermutation::One) => Some(tour.clone()),
            (Some(tour), ThreeOptPermutation::Two) => Some(tour.swap(self.index1, self.index2)),
            (Some(tour), ThreeOptPermutation::Three) => Some(tour.swap(self.index1, self.index3)),
            (Some(tour), ThreeOptPermutation::Four) => Some(tour.swap(self.index2, self.index3)),
            _ => panic!("invalid state"),
        }
    }
}

impl Operator<Tour> for ThreeOpt {
    fn construct_neighborhood(&self, solution: Tour) -> Box<dyn Iterator<Item = Tour>> {
        let mut neighborhood = Self::new(self.cities.as_ref());
        neighborhood.tour = Some(solution);
        Box::new(neighborhood)
    }
}

impl City {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

impl Tour {
    fn new(cities: Vec<City>) -> Self {
        Self { cities }
    }

    // fn len(&self) -> usize {
    //     self.cities.len()
    // }

    // fn get(&self, index: usize) -> Option<&City> {
    //     self.cities.get(index)
    // }

    fn swap(&self, index1: usize, index2: usize) -> Tour {
        let mut solution = self.clone();
        solution.cities.swap(index1, index2);
        solution
    }
}

impl Evaluate for Tour {
    fn evaluate(&self) -> f32 {
        if self.cities.is_empty() {
            return 0.;
        }

        let mut sum = 0.;

        for i in 0..self.cities.len() - 1 {
            let ref city1 = self.cities[i];
            let ref city2 = self.cities[i + 1];
            sum += distance(city1, city2);
        }

        -sum
    }
}

fn construct_greedy_tour(cities: &mut dyn Iterator<Item = City>, rng: &mut dyn RngCore) -> Tour {
    let mut cities: Vec<City> = cities.collect();
    let index_initial_city = rng.gen_range(0..cities.len());
    let city = cities.remove(index_initial_city);
    let mut cities_tour = vec![city.clone()];

    while !cities.is_empty() {
        let city = remove_closest_city(&city, &mut cities);
        cities_tour.push(city);
    }

    Tour::new(cities_tour)
}

fn construct_random_tour(cities: &mut dyn Iterator<Item = City>, rng: &mut dyn RngCore) -> Tour {
    let mut cities: Vec<City> = cities.collect();
    let mut cities_tour = vec![];

    while !cities.is_empty() {
        let index_city = rng.gen_range(0..cities.len());
        let city = cities.remove(index_city);
        cities_tour.push(city.clone());
    }

    Tour::new(cities_tour)
}

fn remove_closest_city(reference_city: &City, cities: &mut Vec<City>) -> City {
    let distances = cities.iter().map(|city| distance(city, reference_city));

    let mut iter = distances.enumerate();
    let init = iter.next().unwrap();

    let (index_closest, _) = iter.fold(init, |(index_accum, distance_accum), (index, distance)| {
        if distance < distance_accum {
            (index, distance)
        } else {
            (index_accum, distance_accum)
        }
    });

    cities.remove(index_closest)
}

fn create_random_city(width: f32, height: f32, rng: &mut dyn rand::RngCore) -> City {
    let w = rng.gen::<f32>() * width;
    let h = rng.gen::<f32>() * height;
    City::new(w, h)
}

fn distance(city1: &City, city2: &City) -> f32 {
    let delta_x = (city1.x() - city2.x()).abs();
    let delta_y = (city1.y() - city2.y()).abs();

    (delta_x.powf(2.) + delta_y.powf(2.)).sqrt()
}
