#[derive(Debug)]
pub struct FileBlockMetadata {
    t: String,
    index: usize,
}

impl FileBlockMetadata {
    pub fn new(t: String, index: usize) -> FileBlockMetadata {
        FileBlockMetadata {
            t,
            index,
        }
    }

    pub fn t(&self) -> String {
        self.t.clone()
    }

    pub fn index(&self) -> usize {
        self.index
    }
}