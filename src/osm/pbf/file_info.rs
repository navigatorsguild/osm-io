use crate::osm::model::bounding_box::BoundingBox;

#[derive(Debug, Default, Clone)]
pub struct FileInfo {
    pub bounding_box: Option<BoundingBox>,
    pub required_features: Vec<String>,
    pub optional_features: Vec<String>,
    pub writingprogram: Option<String>,
    pub source: Option<String>,
    pub osmosis_replication_timestamp: Option<i64>,
    pub osmosis_replication_sequence_number: Option<i64>,
    pub osmosis_replication_base_url: Option<String>,
}


impl FileInfo {
    pub fn new(
        bounding_box: Option<BoundingBox>,
        required_features: Vec<String>,
        optional_features: Vec<String>,
        writingprogram: Option<String>,
        source: Option<String>,
        osmosis_replication_timestamp: Option<i64>,
        osmosis_replication_sequence_number: Option<i64>,
        osmosis_replication_base_url: Option<String>,
    ) -> Self {
        FileInfo {
            bounding_box,
            required_features,
            optional_features,
            writingprogram,
            source,
            osmosis_replication_timestamp,
            osmosis_replication_sequence_number,
            osmosis_replication_base_url,
        }
    }

    pub(crate) fn merge_bounding_box(&mut self, bounding_box: Option<BoundingBox>) {
        if self.bounding_box.is_none() {
            self.bounding_box = bounding_box;
        } else {
            if bounding_box.is_some() {
                self.bounding_box.as_mut().unwrap().merge_bounding_box(bounding_box.as_ref().unwrap());
            }
        }
    }
}