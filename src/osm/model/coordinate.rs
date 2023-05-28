#[derive(Debug, Clone)]
pub struct Coordinate {
    lat: f64,
    lon: f64,
}

impl Coordinate {
    pub fn new(lat: f64, lon: f64) -> Coordinate {
        Coordinate {
            lat,
            lon,
        }
    }

    pub fn lat(&self) -> f64 {
        self.lat
    }

    pub fn lat7(&self) -> i64 {
        (self.lat * 1E7) as i64
    }

    pub fn lon(&self) -> f64 {
        self.lon
    }

    pub fn lon7(&self) -> i64 {
        (self.lon * 1E7) as i64
    }
}