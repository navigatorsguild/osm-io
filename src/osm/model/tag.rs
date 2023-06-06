#[derive(Debug, Clone)]
pub struct Tag {
    k: String,
    v: String,
}

impl Tag {
    pub fn new(k: String, v: String) -> Tag {
        Tag {
            k,
            v,
        }
    }

    pub fn k(&self) -> &String {
        &self.k
    }

    pub fn v(&self) -> &String {
        &self.v
    }
}