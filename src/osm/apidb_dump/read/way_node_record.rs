#[derive(Debug)]
pub(crate) struct WayNodeRecord {
    way_id: i64,
    node_id: i64,
    version: i64,
    sequence_id: i64,
}

impl WayNodeRecord {
    pub(crate) fn new(
        way_id: i64,
        node_id: i64,
        version: i64,
        sequence_id: i64,
    )-> WayNodeRecord {
        WayNodeRecord {
            way_id,
            node_id,
            version,
            sequence_id,
        }
    }

    pub(crate)  fn way_id(&self) -> i64 {
        self.way_id
    }

    pub(crate)  fn node_id(&self) -> i64 {
        self.node_id
    }
    pub(crate)  fn version(&self) -> i64 {
        self.version
    }

    pub(crate)  fn sequence_id(&self) -> i64 {
        self.sequence_id
    }
}