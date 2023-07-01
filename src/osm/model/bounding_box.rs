use std::fmt::{Display, Formatter};
use std::str::FromStr;

use anyhow::anyhow;

use crate::osm::model::coordinate::Coordinate;

#[derive(Debug, Clone)]
pub struct BoundingBox {
    left: f64,
    bottom: f64,
    right: f64,
    top: f64,
}

impl BoundingBox {
    pub fn new(left: f64, bottom: f64, right: f64, top: f64) -> BoundingBox {
        BoundingBox {
            left,
            bottom,
            right,
            top,
        }
    }

    pub fn merge_point(&mut self, coordinate: &Coordinate) {
        if coordinate.lon() < self.left {
            self.left = coordinate.lon();
        }

        if coordinate.lon() > self.right {
            self.right = coordinate.lon();
        }

        if coordinate.lat() > self.top {
            self.top = coordinate.lat();
        }

        if coordinate.lat() < self.bottom {
            self.bottom = coordinate.lat();
        }
    }

    pub fn merge_bounding_box(&mut self, other: &BoundingBox) {
        if other.left < self.left {
            self.left = other.left;
        }

        if other.right > self.right {
            self.right = other.right;
        }

        if other.top > self.top {
            self.top = other.top;
        }

        if other.bottom < self.bottom {
            self.bottom = other.bottom;
        }
    }

    pub fn left(&self) -> f64 {
        self.left
    }

    pub fn right(&self) -> f64 {
        self.right
    }

    pub fn top(&self) -> f64 {
        self.top
    }

    pub fn bottom(&self) -> f64 {
        self.bottom
    }
}

impl Display for BoundingBox {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "left: {}, bottom: {}, right: {}, top: {}", self.left, self.bottom, self.right, self.top)
    }
}

impl FromStr for BoundingBox {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let e = anyhow!("Bounding box string must be in the form of 'left,bottom,right,top' as in -180.0, -90.0, 180.0, 90.0 with optional white space around commas. Got {} instead", s);
        let parts: Vec<&str> = s.split(",")
            .map(|s| s.trim())
            .collect();
        if parts.len() < 4 {
            Err(e)
        } else {
            let left = f64::from_str(parts[0])?;
            let bottom = f64::from_str(parts[1])?;
            let right = f64::from_str(parts[2])?;
            let top = f64::from_str(parts[3])?;
            if left > 180.0 || left < -180.0
                || bottom > 90.0 || bottom < -90.0
                || right > 180.0 || right < -180.0
                || top > 90.0 || top < -90.0
            {
                Err(e)
            } else {
                Ok(BoundingBox::new(left, bottom, right, top))
            }
        }
    }
}

impl Default for BoundingBox {
    fn default() -> Self {
        BoundingBox::new(-180.0, -90.0, 180.0, 90.0)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::osm::model::bounding_box::BoundingBox;

    #[test]
    fn test_from_str() -> Result<(), anyhow::Error> {
        let bounding_box = BoundingBox::from_str("-180.0, -90.0, 180.0, 90.0")?;
        assert_eq!(bounding_box.left, -180.0);
        assert_eq!(bounding_box.bottom, -90.0);
        assert_eq!(bounding_box.right, 180.0);
        assert_eq!(bounding_box.top, 90.0);
        Ok(())
    }

    #[test]
    #[should_panic]
    fn test_invalid_values() {
        BoundingBox::from_str("-180.1, -90.0, 180.0, 90.0").unwrap();
    }
}
