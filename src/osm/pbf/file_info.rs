use crate::osm::model::bounding_box::BoundingBox;

/// *.osm.pbf header data
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
    /// Prepare OSM header data
    ///
    /// Example:
    /// ```
    /// use osm_io::osm::pbf::file_info::FileInfo;
    /// let file_info = FileInfo::default()
    ///     .with_writingprogram_str("example-osm-pbf-writer");
    /// ```
    #[allow(clippy::too_many_arguments)]
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
        } else if bounding_box.is_some() {
            self.bounding_box.as_mut().unwrap().merge_bounding_box(bounding_box.as_ref().unwrap());
        }
    }

    /// Get the bounding box for the data in this file
    pub fn bounding_box(&self) -> &Option<BoundingBox> {
        &self.bounding_box
    }

    /// Set the bounding box for the data in this file
    pub fn with_bounding_box(&mut self, bounding_box: &Option<BoundingBox>) {
        self.bounding_box = bounding_box.clone();
    }

    /// Get required features for this file
    pub fn required_features(&self) -> &Vec<String> {
        &self.required_features
    }

    /// Set required features for this file
    pub fn with_required_features(&mut self, required_features: &[String]) {
        self.required_features = required_features.to_vec();
    }

    /// Get optional features for this file
    pub fn optional_features(&self) -> &Vec<String> {
        &self.optional_features
    }

    /// Set optional features for this file
    pub fn with_optional_features(&mut self, optional_features: &[String]) {
        self.optional_features = optional_features.to_vec();
    }

    /// Get writing program set for this file
    pub fn writingprogram(&self) -> &Option<String> {
        &self.writingprogram
    }

    /// Set writing program for this file
    pub fn with_writingprogram(&mut self, writingprogram: &Option<String>) {
        self.writingprogram = writingprogram.clone();
    }

    /// As with_writingprogram above but accept &str
    pub fn with_writingprogram_str(&mut self, writingprogram: &str) {
        self.writingprogram = Some(writingprogram.to_string())
    }

    /// Get the source set for this file
    pub fn source(&self) -> &Option<String> {
        &self.source
    }

    /// Set the source for this file
    pub fn with_source(&mut self, source: &Option<String>) {
        self.source = source.clone();
    }

    /// Get the osmosis_replication_timestamp set for this file
    pub fn osmosis_replication_timestamp(&self) -> &Option<i64> {
        &self.osmosis_replication_timestamp
    }

    /// Set the osmosis_replication_timestamp for this file
    pub fn with_osmosis_replication_timestamp(&mut self, osmosis_replication_timestamp: &Option<i64>) {
        self.osmosis_replication_timestamp = *osmosis_replication_timestamp;
    }

    /// Get osmosis_replication_sequence_number set for this file
    pub fn osmosis_replication_sequence_number(&self) -> &Option<i64> {
        &self.osmosis_replication_sequence_number
    }

    /// Set osmosis_replication_sequence_number for this file
    pub fn with_osmosis_replication_sequence_number(&mut self, osmosis_replication_sequence_number: &Option<i64>) {
        self.osmosis_replication_sequence_number = *osmosis_replication_sequence_number;
    }

    /// Get osmosis_replication_base_url set for this file
    pub fn osmosis_replication_base_url(&self) -> &Option<String> {
        &self.osmosis_replication_base_url
    }

    /// Set osmosis_replication_base_url for this file
    pub fn with_osmosis_replication_base_url(&mut self, osmosis_replication_base_url: &Option<String>) {
        self.osmosis_replication_base_url = osmosis_replication_base_url.clone();
    }

    pub fn required(&self, feature: &str) -> bool {
        self.required_features.contains(&feature.to_string())
    }

    pub fn optional(&self, feature: &str) -> bool {
        self.optional_features.contains(&feature.to_string())
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