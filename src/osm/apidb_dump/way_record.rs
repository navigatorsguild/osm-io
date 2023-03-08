use chrono::NaiveDateTime;

#[derive(Debug)]
pub(crate) struct WayRecord {
    way_id: i64,
    changeset_id: i64,
    timestamp: NaiveDateTime,
    version: i64,
    visible: bool,
    redaction_id: Option<i32>,
}

impl WayRecord {
    pub(crate) fn new(
        way_id: i64,
        changeset_id: i64,
        timestamp: NaiveDateTime,
        version: i64,
        visible: bool,
        redaction_id: Option<i32>,
    ) -> WayRecord {
        WayRecord {
            way_id,
            changeset_id,
            timestamp,
            version,
            visible,
            redaction_id,
        }
    }

    pub(crate)  fn way_id(&self) -> i64 {
        self.way_id
    }

    pub(crate)  fn changeset_id(&self) -> i64 {
        self.changeset_id
    }

    pub(crate)  fn timestamp(&self) -> NaiveDateTime {
        self.timestamp
    }

    pub(crate)  fn version(&self) -> i64 {
        self.version
    }

    pub(crate)  fn visible(&self) -> bool {
        self.visible
    }

    pub(crate)  fn redaction_id(&self) -> Option<i32> {
        self.redaction_id
    }
}