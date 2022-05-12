fn main() {}

struct City {
    id: usize,
    x: f32,
    y: f32,
}

impl City {
    fn new(id: usize, x: f32, y: f32) -> Self {
        Self { id, x, y }
    }
}

struct Tour {
    cities: Vec<City>,
}

impl Tour {
    fn new(cities: Vec<City>) -> Self {
        Self { cities }
    }

    fn push(&mut self, city: City) {
        self.cities.push(city)
    }

    fn sequence(&self) -> impl Iterator<Item = &City> {
        self.cities.iter()
    }
}
