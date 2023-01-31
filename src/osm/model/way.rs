use crate::osm::model::tag::Tag;

#[derive(Debug)]
pub struct Way {
    id: i64,
    timestamp: i64,
    changeset: i64,
    uid: i32,
    user: String,
    visible: bool,
    refs: Vec<i64>,
    tags: Vec<Tag>,

    // TODO: LocationsOnWays
}

impl Way {
    pub fn new(id: i64, timestamp: i64, changeset: i64, uid: i32, user: String, visible: bool, refs: Vec<i64>, tags: Vec<Tag>) -> Way {
        Way {
            id,
            timestamp,
            changeset,
            uid,
            user,
            visible,
            refs,
            tags,
        }
    }
}