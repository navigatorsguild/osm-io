use chrono::NaiveDateTime;

#[derive(Debug)]
pub(crate) struct NodeRecord {
    node_id: i64,
    latitude: i32,
    longitude: i32,
    changeset_id: i64,
    visible: bool,
    timestamp: NaiveDateTime,
    tile: i64,
    version: i64,
    redaction_id: Option<i32>,
}

#[allow(clippy::too_many_arguments)]
impl NodeRecord {
    pub(crate) fn new(
        node_id: i64,
        latitude: i32,
        longitude: i32,
        changeset_id: i64,
        visible: bool,
        timestamp: NaiveDateTime,
        tile: i64,
        version: i64,
        redaction_id: Option<i32>,
    ) -> NodeRecord {
        NodeRecord {
            node_id,
            latitude,
            longitude,
            changeset_id,
            visible,
            timestamp,
            tile,
            version,
            redaction_id,
        }
    }

    pub(crate) fn node_id(&self) -> i64 {
        self.node_id
    }

    pub(crate) fn latitude(&self) -> i32 {
        self.latitude
    }

    pub(crate) fn longitude(&self) -> i32 {
        self.longitude
    }

    pub(crate) fn changeset_id(&self) -> i64 {
        self.changeset_id
    }

    pub(crate) fn visible(&self) -> bool {
        self.visible
    }

    pub(crate) fn timestamp(&self) -> NaiveDateTime {
        self.timestamp
    }

    #[allow(dead_code)]
    pub(crate) fn tile(&self) -> i64 {
        self.tile
    }

    pub(crate) fn version(&self) -> i64 {
        self.version
    }

    #[allow(dead_code)]
    pub(crate) fn redaction_id(&self) -> Option<i32> {
        self.redaction_id
    }
}
