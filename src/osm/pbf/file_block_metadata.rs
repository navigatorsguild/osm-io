use crate::osm::model::bounding_box::BoundingBox;

#[derive(Debug, Default, Clone)]
pub struct FileBlockMetadata {
    t: String,
    index: usize,
    bounding_box: Option<BoundingBox>,
}

impl FileBlockMetadata {
    pub(crate) fn new(t: String, index: usize) -> FileBlockMetadata {
        FileBlockMetadata {
            t,
            index,
            bounding_box: None,
        }
    }

    #[allow(dead_code)]
    pub(crate) fn t(&self) -> String {
        self.t.clone()
    }

    #[allow(dead_code)]
    pub(crate) fn index(&self) -> usize {
        self.index
    }

    #[allow(dead_code)]
    pub(crate) fn bounding_box(&self) -> Option<BoundingBox> {
        self.bounding_box.clone()
    }

    #[allow(dead_code)]
    pub(crate) fn set_bounding_box(&mut self, bounding_box: BoundingBox) {
        self.bounding_box.replace(bounding_box);
    }
}