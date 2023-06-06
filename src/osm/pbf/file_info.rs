use crate::osm::model::bounding_box::BoundingBox;

#[derive(Debug, Clone)]
pub struct FileInfo {
    bounding_box: Option<BoundingBox>,
    required_features: Vec<String>,
    optional_features: Vec<String>,
    writingprogram: Option<String>,
    source: Option<String>,
    osmosis_replication_timestamp: Option<i64>,
    osmosis_replication_sequence_number: Option<i64>,
    osmosis_replication_base_url: Option<String>,
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

    #[allow(dead_code)]
    pub(crate) fn merge_bounding_box(&mut self, bounding_box: Option<BoundingBox>) {
        if self.bounding_box.is_none() {
            self.bounding_box = bounding_box;
        } else {
            if bounding_box.is_some() {
                self.bounding_box.as_mut().unwrap().merge_bounding_box(bounding_box.as_ref().unwrap());
            }
        }
    }

    pub fn bounding_box(&self) -> &Option<BoundingBox> {
        &self.bounding_box
    }

    pub fn with_bounding_box(&mut self, bounding_box: &Option<BoundingBox>) {
        self.bounding_box = bounding_box.clone();
    }

    pub fn required_features(&self) -> &Vec<String> {
        &self.required_features
    }

    pub fn with_required_features(&mut self, required_features: &Vec<String>) {
        self.required_features = required_features.clone();
    }

    pub fn optional_features(&self) -> &Vec<String> {
        &self.optional_features
    }

    pub fn with_optional_features(&mut self, optional_features: &Vec<String>) {
        self.optional_features = optional_features.clone();
    }

    pub fn writingprogram(&self) -> &Option<String> {
        &self.writingprogram
    }

    pub fn with_writingprogram(&mut self, writingprogram: &Option<String>) {
        self.writingprogram = writingprogram.clone();
    }

    pub fn with_writingprogram_str(&mut self, writingprogram: &str) {
        self.writingprogram = Some(writingprogram.to_string())
    }

    pub fn source(&self) -> &Option<String> {
        &self.source
    }

    pub fn with_source(&mut self, source: &Option<String>) {
        self.source = source.clone();
    }

    pub fn osmosis_replication_timestamp(&self) -> &Option<i64> {
        &self.osmosis_replication_timestamp
    }

    pub fn with_osmosis_replication_timestamp(&mut self, osmosis_replication_timestamp: &Option<i64>) {
        self.osmosis_replication_timestamp = osmosis_replication_timestamp.clone();
    }

    pub fn osmosis_replication_sequence_number(&self) -> &Option<i64> {
        &self.osmosis_replication_sequence_number
    }

    pub fn with_osmosis_replication_sequence_number(&mut self, osmosis_replication_sequence_number: &Option<i64>) {
        self.osmosis_replication_sequence_number = osmosis_replication_sequence_number.clone();
    }

    pub fn osmosis_replication_base_url(&self) -> &Option<String> {
        &self.osmosis_replication_base_url
    }

    pub fn with_osmosis_replication_base_url(&mut self, osmosis_replication_base_url: &Option<String>) {
        self.osmosis_replication_base_url = osmosis_replication_base_url.clone();
    }
}

impl Default for FileInfo {
    fn default() -> Self {
        FileInfo::new(
            None,
            ["OsmSchema-V0.6", "DenseNodes"].map(|s| s.to_string()).to_vec(),
            ["Sort.Type_then_ID"].map(|s| s.to_string()).to_vec(),
            None,
            None,
            None,
            None,
            None,
        )
    }
}