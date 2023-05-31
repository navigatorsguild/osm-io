use chrono::NaiveDateTime;

#[derive(Debug)]
pub(crate) struct RelationRecord {
    relation_id: i64,
    changeset_id: i64,
    timestamp: NaiveDateTime,
    version: i64,
    visible: bool,
    redaction_id: Option<i32>,
}

impl RelationRecord {
    pub(crate) fn new(
        relation_id: i64,
        changeset_id: i64,
        timestamp: NaiveDateTime,
        version: i64,
        visible: bool,
        redaction_id: Option<i32>,
    ) -> RelationRecord {
        RelationRecord {
            relation_id,
            changeset_id,
            timestamp,
            version,
            visible,
            redaction_id,
        }
    }

    pub(crate)  fn relation_id(&self) -> i64 {
        self.relation_id
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

    #[allow(dead_code)]
    pub(crate)  fn redaction_id(&self) -> Option<i32> {
        self.redaction_id
    }
}