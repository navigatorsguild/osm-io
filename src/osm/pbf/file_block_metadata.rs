use crate::osm::model::bounding_box::BoundingBox;

#[derive(Debug, Default, Clone)]
pub struct FileBlockMetadata {
    t: String,
    index: usize,
    bounding_box: Option<BoundingBox>,
}

impl FileBlockMetadata {
    pub fn new(t: String, index: usize) -> FileBlockMetadata {
        FileBlockMetadata {
            t,
            index,
            bounding_box: None,
        }
    }

    pub fn t(&self) -> String {
        self.t.clone()
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn bounding_box(&self) -> Option<BoundingBox> {
        self.bounding_box.clone()
    }

    pub fn set_bounding_box(&mut self, bounding_box: BoundingBox) {
        self.bounding_box.replace(bounding_box);
    }

    pub fn is_sentinel(&self) -> bool {
        match self.t.as_str() {
            "Sentinel" => {
                true
            }
            _ => {
                false
            }
        }
    }
}