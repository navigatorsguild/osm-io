#[derive(Debug)]
pub struct Coordinate {
    pub lat: f64,
    pub lon: f64,
}

impl Coordinate {
    pub fn new(lat: f64, lon: f64) -> Coordinate {
        Coordinate {
            lat,
            lon,
        }
    }
}