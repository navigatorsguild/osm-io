use chrono::NaiveDateTime;

#[derive(Debug)]
pub(crate) struct ChangesetRecord {
    id: i64,
    user_id: i64,
    created_at: NaiveDateTime,
    min_lat: i32,
    max_lat: i32,
    min_lon: i32,
    max_lon: i32,
    closed_at: NaiveDateTime,
    num_changes: i32,
}

impl ChangesetRecord {
    pub(crate) fn new(
        id: i64,
        user_id: i64,
        created_at: NaiveDateTime,
        min_lat: i32,
        max_lat: i32,
        min_lon: i32,
        max_lon: i32,
        closed_at: NaiveDateTime,
        num_changes: i32,
    ) -> ChangesetRecord {
        ChangesetRecord {
            id,
            user_id,
            created_at,
            min_lat,
            max_lat,
            min_lon,
            max_lon,
            closed_at,
            num_changes,
        }
    }

    pub(crate) fn id(&self) -> i64 {
        self.id
    }

    pub(crate) fn user_id(&self) -> i64 {
        self.user_id
    }

    pub(crate) fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }

    pub(crate) fn min_lat(&self) -> i32 {
        self.min_lat
    }

    pub(crate) fn max_lat(&self) -> i32 {
        self.max_lat
    }

    pub(crate) fn min_lon(&self) -> i32 {
        self.min_lon
    }

    pub(crate) fn max_lon(&self) -> i32 {
        self.max_lon
    }

    pub(crate) fn closed_at(&self) -> NaiveDateTime {
        self.closed_at
    }

    pub(crate) fn num_changes(&self) -> i32 {
        self.num_changes
    }
}
