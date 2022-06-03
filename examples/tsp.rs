use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    time::{Duration, SystemTime},
};

use netaheuristics::{
    algorithms::{
        lns::LargeNeighborhoodSearch, sa::SimulatedAnnealing, vns::VariableNeighborhoodSearch,
    },
    selectors::{AdaptiveSelector, RandomSelector, SequentialSelector},
    termination::{Terminator, TimeTerminator},
    Evaluate, ImprovingHeuristic, Operator, Outcome,
};
use rand::{Rng, RngCore, SeedableRng};

fn main() {
    // init
    let n = 10;
    let width = 100.;
    let height = 100.;
    let computation_time_max = Duration::new(2, 0);

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
    let random_outcome = Outcome::new(random_tour, duration_random);
    let now = SystemTime::now();
    let greedy_tour = construct_greedy_tour(&mut cities.clone().into_iter(), &mut rng);
    let duration_greedy = now.elapsed().unwrap();
    let greedy_outcome = Outcome::new(greedy_tour, duration_greedy);

    // optimize with VNS
    let operator1 = TwoOpt::new(cities.as_slice());
    let operator2 = Insertion::new(cities.as_slice());
    let vns = VariableNeighborhoodSearch::builder()
        .selector(
            SequentialSelector::new()
                .option(operator1)
                .option(operator2),
        )
        .terminator(TimeTerminator::new(computation_time_max))
        .build();
    let vns_outcome = vns.optimize_timed(random_outcome.solution().clone());

    // optimize with Simulated Annealing
    let temperature = 100.;
    let operator = TwoOpt::new(cities.as_slice());
    let sa = SimulatedAnnealing::builder()
        .selector(RandomSelector::new(rng.clone()).option(operator))
        .operator(TwoOptRandom)
        .temperature(temperature)
        .terminator(
            Terminator::builder()
                .computation_time(computation_time_max)
                .build(),
        )
        .rng(rng.clone())
        .build();
    let sa_outcome = sa.optimize_timed(random_outcome.solution().clone());

    // optimize with Large Neighborhood Search
    let n_destroyed_cities = 2;
    let destroyer = TSPDestroyer::new(n_destroyed_cities);
    let repairer = TSPRepairer::new(*cities.clone());
    let lns = LargeNeighborhoodSearch::builder()
        .selector_destroyer(SequentialSelector::new().option(destroyer))
        .selector_repairer(SequentialSelector::new().option(repairer))
        .terminator(
            Terminator::builder()
                .computation_time(computation_time_max)
                .build(),
        )
        .rng(rng.clone())
        .build();
    let lns_outcome = lns.optimize_timed(random_outcome.solution().clone());

    // optimize with adaptive VNS
    let decay = 0.5;
    let operator1 = TwoOpt::new(cities.as_slice());
    let operator2 = Insertion::new(cities.as_slice());
    let adaptive_vns = VariableNeighborhoodSearch::builder()
        .selector(
            AdaptiveSelector::default_weights(decay, rng)
                .operator(operator1)
                .operator(operator2),
        )
        .terminator(TimeTerminator::new(computation_time_max))
        .build();
    let adaptive_vns_outcome = adaptive_vns.optimize_timed(random_outcome.solution().clone());

    // display results
    show_solution(random_outcome, "random");
    show_solution(greedy_outcome, "greedy");
    show_solution(vns_outcome, "vns");
    show_solution(adaptive_vns_outcome, "adaptive vns");
    show_solution(sa_outcome, "sa");
    show_solution(lns_outcome, "lns");
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

struct Insertion {
    tour: Option<Tour>,
    cities: Box<Vec<City>>,
    index1: usize,
    index2: usize,
}

struct TSPDestroyer {
    n: usize,
}

struct TSPRepairer {
    cities: Vec<City>,
}

fn show_solution<Solution: Evaluate>(outcome: Outcome<Solution>, method: &str) {
    println!(
        "{} tour length: {}, computation time: {}",
        method,
        outcome.solution().evaluate(),
        outcome.duration().as_nanos() as f32 * 1e-9
    );
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

impl Operator for TSPRepairer {
    type Solution = Tour;
    fn shake(&self, mut solution: Self::Solution, _rng: &mut dyn rand::RngCore) -> Self::Solution {
        let map: HashMap<City, usize> = self
            .cities
            .iter()
            .enumerate()
            .map(|(index, city)| (city.clone(), index))
            .collect();
        let cities: HashSet<City> = self.cities.iter().map(|x| x.to_owned()).collect();
        let cities_tour: HashSet<City> = solution.cities.clone().into_iter().collect();
        let cities_missing: HashSet<City> = &cities - &cities_tour;
        let mut cities_missing: Vec<City> = cities_missing.into_iter().collect();
        cities_missing.sort_by(|x, y| {
            let index_x = map[x];
            let index_y = map[y];
            index_x.cmp(&index_y)
        });

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

impl Operator for TSPDestroyer {
    type Solution = Tour;
    fn shake(&self, mut solution: Self::Solution, rng: &mut dyn rand::RngCore) -> Self::Solution {
        for _ in 0..self.n {
            let r = rng.gen_range(0..solution.cities.len());
            solution.cities.remove(r);
        }
        solution
    }
}

impl Operator for TwoOptRandom {
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

        if self.index1 == self.index2 {
            self.advance()
        } else {
            true
        }
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

    fn shake(&self, solution: Self::Solution, rng: &mut dyn rand::RngCore) -> Self::Solution {
        let n = solution.cities.len();
        let index1 = rng.gen_range(0..n);
        let index2 = rng.gen_range(0..n);

        let mut neighbor = solution.clone();
        neighbor.cities.swap(index1, index2);
        neighbor
    }
}

impl Insertion {
    fn new(cities: &[City]) -> Self {
        Self {
            tour: None,
            cities: Box::new(cities.to_owned()),
            index1: 0,
            index2: 1,
        }
    }

    // advance index3, index2, index1
    fn advance(&mut self) -> bool {
        // init
        let number_cities = self.cities.len();

        // index1 locked, index2 locked
        if self.index1 == number_cities - 1 && self.index2 == number_cities - 1 {
            return false;
        }
        // index1 unlocked, index2 locked
        else if self.index1 < number_cities - 1 && self.index2 == number_cities - 1 {
            self.index1 += 1;
            self.index2 = 0;
        }
        // index 1 ?, index2 unlocked
        else {
            self.index2 += 1;
        }

        if self.index1 == self.index2 {
            self.advance()
        } else {
            true
        }
    }
}

impl Iterator for Insertion {
    type Item = Tour;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.advance() {
            return None;
        }

        if let Some(tour) = &self.tour {
            let tour = tour.reinsert(self.index1, self.index2);
            return Some(tour);
        } else {
            None
        }
    }
}

impl Operator for Insertion {
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

    fn reinsert(&self, from: usize, to: usize) -> Tour {
        let mut solution = self.clone();
        let city = solution.cities.remove(from);
        solution.cities.insert(to, city);
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

// todo: fix 3-opt. seems to be the same as 2-opt, takes very long
