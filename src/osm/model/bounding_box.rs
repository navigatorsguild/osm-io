#[derive(Debug, Default, Clone)]
pub struct BoundingBox{
    pub left: f64,
    pub right: f64,
    pub top: f64,
    pub bottom: f64,
}

impl BoundingBox {
    pub fn new(left: f64, right: f64, top: f64, bottom: f64) -> BoundingBox {
        BoundingBox {
            left,
            right,
            top,
            bottom,
        }
    }
}