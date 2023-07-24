#[derive(Debug, Clone, PartialEq)]
pub struct Coordinate {
    lat: f64,
    lon: f64,
}

impl Coordinate {
    pub fn new(lat: f64, lon: f64) -> Coordinate {
        // history files contain invalid coordinates for nodes
        // assert!(lon >= -180.0 && lon <= 180.0);
        // assert!(lat >= -90.0 && lat <= 90.0);
        Coordinate {
            lat,
            lon,
        }
    }

    pub fn lat(&self) -> f64 {
        self.lat
    }

    pub fn lat7(&self) -> i64 {
        (self.lat * 1E7).round() as i64
    }

    pub fn lon(&self) -> f64 {
        self.lon
    }

    pub fn lon7(&self) -> i64 {
        (self.lon * 1E7).round() as i64
    }
}
