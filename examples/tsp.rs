use std::{collections::HashSet, hash::Hash, time::SystemTime};

use heuristics::{
    lns::{Destroyer, LargeNeighborhoodSearch, Repairer},
    sa::SimulatedAnnealing,
    termination::Terminator,
    vns::VariableNeighborhoodSearch,
    Evaluate, ImprovingHeuristic, Operator, RandomSelector, SequentialSelector, StochasticOperator,
};
use rand::{Rng, RngCore, SeedableRng};

fn main() {
    // init
    let n = 10;
    let width = 100.;
    let height = 100.;

    // create random cities
    let seed = 0;
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let cities: Vec<City> = (0..n)
        .map(|id| create_random_city(id, width, height, &mut rng))
        .collect();
    let cities = Box::new(cities);

    let now = SystemTime::now();
    let random_tour = construct_random_tour(&mut cities.clone().into_iter(), &mut rng);
    let duration_random = now.elapsed().unwrap();
    let now = SystemTime::now();
    let greedy_tour = construct_greedy_tour(&mut cities.clone().into_iter(), &mut rng);
    let duration_greedy = now.elapsed().unwrap();

    // optimize with VNS
    let operator_2opt = TwoOpt::new(cities.as_slice());
    let operator_3opt = ThreeOpt::new(cities.as_slice());

    let vns = VariableNeighborhoodSearch::builder()
        .selector(SequentialSelector::new(2))
        .operator(operator_2opt)
        .operator(operator_3opt)
        .terminator(Terminator::builder().iterations(10).build())
        .build();
    let vns_outcome = vns.optimize_timed(random_tour.clone());

    let seed = 0;
    let rng = rand::rngs::StdRng::seed_from_u64(seed);
    let temperature = 100.;
    let n_iterations = 40_000;
    let sa = SimulatedAnnealing::builder()
        .selector(RandomSelector::new(rng.clone(), 1))
        .operator(TwoOptRandom)
        .temperature(temperature)
        .terminator(Terminator::builder().iterations(n_iterations).build())
        .rng(rng)
        .build();
    let sa_outcome = sa.optimize_timed(random_tour.clone());

    let seed = 0;
    let rng = rand::rngs::StdRng::seed_from_u64(seed);
    let n_destroyed_cities = 2;
    let lns = LargeNeighborhoodSearch::builder()
        .selector_destroyer(SequentialSelector::new(1))
        .selector_repairer(SequentialSelector::new(1))
        .destroyer(TSPDestroyer::new(n_destroyed_cities))
        .repairer(TSPRepairer::new(*cities))
        .terminator(Terminator::builder().iterations(10_000).build())
        .rng(rng)
        .build();
    let lns_outcome = lns.optimize_timed(random_tour.clone());

    println!(
        "random tour length: {}, computation time: {}",
        random_tour.evaluate(),
        duration_random.as_nanos() as f32 * 1e-9
    );
    println!(
        "greedy tour length: {}, computation time: {}",
        greedy_tour.evaluate(),
        duration_greedy.as_nanos() as f32 * 1e-9
    );
    println!(
        "vns tour length: {}, computation time: {}",
        vns_outcome.solution().evaluate(),
        vns_outcome.duration().as_nanos() as f32 * 1e-9
    );
    println!(
        "sa tour length: {}, computation time: {}",
        sa_outcome.solution().evaluate(),
        sa_outcome.duration().as_nanos() as f32 * 1e-9
    );
    println!(
        "lns tour length: {}, computation time: {}",
        lns_outcome.solution().evaluate(),
        lns_outcome.duration().as_nanos() as f32 * 1e-9
    );
}

#[derive(Clone, Debug)]
struct City {
    id: usize,
    x: f32,
    y: f32,
}

#[derive(Clone, Debug)]
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

struct TwoOptRandom;

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

struct TSPDestroyer {
    n: usize,
}

struct TSPRepairer {
    cities: Vec<City>,
}

impl TSPRepairer {
    fn new(cities: Vec<City>) -> Self {
        Self { cities }
    }
}

impl TSPDestroyer {
    pub fn new(n: usize) -> Self {
        Self { n }
    }
}

impl Repairer for TSPRepairer {
    type Solution = Tour;
    fn repair(&self, mut solution: Self::Solution, _rng: &mut dyn rand::RngCore) -> Self::Solution {
        let cities: HashSet<City> = self.cities.iter().map(|x| x.to_owned()).collect();
        let cities_tour: HashSet<City> = solution.cities.clone().into_iter().collect();
        let cities_missing = &cities - &cities_tour;

        for city in cities_missing {
            let index_to_place = closest_city_to(&city, &solution.cities);
            solution.cities.insert(index_to_place, city);
        }

        solution
    }
}

fn closest_city_to<'a>(city: &'a City, city_pool: &'a Vec<City>) -> usize {
    let mut city_closest_index = 0;
    let mut distance_minimum = distance(city, &city_pool[0]);
    for i in 1..city_pool.len() {
        let distance = distance(city, &city_pool[i]);
        if distance < distance_minimum {
            distance_minimum = distance;
            city_closest_index = i;
        }
    }
    city_closest_index
}

impl Destroyer for TSPDestroyer {
    type Solution = Tour;
    fn destroy(&self, mut solution: Self::Solution, rng: &mut dyn rand::RngCore) -> Self::Solution {
        for _ in 0..self.n {
            let r = rng.gen_range(0..solution.cities.len());
            solution.cities.remove(r);
        }
        solution
    }
}

impl StochasticOperator for TwoOptRandom {
    type Solution = Tour;
    fn shake(&self, solution: Tour, rng: &mut dyn rand::RngCore) -> Self::Solution {
        let n = solution.cities.len();
        let index1 = rng.gen_range(0..n);
        let index2 = rng.gen_range(0..n);

        let mut neighbor = solution.clone();
        neighbor.cities.swap(index1, index2);
        neighbor
    }
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

impl Operator for TwoOpt {
    type Solution = Tour;
    fn construct_neighborhood(&self, solution: Tour) -> Box<dyn Iterator<Item = Tour>> {
        let mut neighborhood = Self::new(self.cities.as_ref());
        neighborhood.tour = Some(solution.clone());
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

impl Operator for ThreeOpt {
    type Solution = Tour;
    fn construct_neighborhood(&self, solution: Tour) -> Box<dyn Iterator<Item = Tour>> {
        let mut neighborhood = Self::new(self.cities.as_ref());
        neighborhood.tour = Some(solution.clone());
        Box::new(neighborhood)
    }
}

impl City {
    fn new(id: usize, x: f32, y: f32) -> Self {
        Self { id, x, y }
    }

    fn x(&self) -> f32 {
        self.x
    }

    fn y(&self) -> f32 {
        self.y
    }
}

impl Eq for City {}

impl PartialEq for City {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for City {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
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

        sum
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

fn create_random_city(id: usize, width: f32, height: f32, rng: &mut dyn rand::RngCore) -> City {
    let w = rng.gen::<f32>() * width;
    let h = rng.gen::<f32>() * height;
    City::new(id, w, h)
}

fn distance(city1: &City, city2: &City) -> f32 {
    let delta_x = (city1.x() - city2.x()).abs();
    let delta_y = (city1.y() - city2.y()).abs();

    (delta_x.powf(2.) + delta_y.powf(2.)).sqrt()
}
