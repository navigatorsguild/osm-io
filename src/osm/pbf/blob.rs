use std::path::PathBuf;

#[derive(Debug)]
pub struct Blob {
    path: PathBuf,
    index: usize,
    start: u64,
    length: u64,
    t: String,
}

impl Blob {
    pub(crate) fn new(path: PathBuf, index: usize, start: u64, length: u64, t: String) -> Blob {
        Blob {
            path,
            index,
            start,
            length,
            t,
        }
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn start(&self) -> u64 {
        self.start
    }

    pub fn length(&self) -> u64 {
        self.length
    }

    pub fn t(&self) -> String {
        self.t.clone()
    }
}