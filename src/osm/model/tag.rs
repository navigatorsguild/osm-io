#[derive(Debug)]
pub struct Tag {
    pub k: String,
    pub v: String,
}

impl Tag{
    pub fn new(k: String, v: String) -> Tag {
        Tag {
            k,
            v,
        }
    }
}