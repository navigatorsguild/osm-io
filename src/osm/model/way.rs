use crate::osm::model::tag::Tag;

#[derive(Debug)]
pub struct Way {
    id: i64,
    version: i32,
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
    pub fn new(id: i64, version: i32, timestamp: i64, changeset: i64, uid: i32, user: String, visible: bool, refs: Vec<i64>, tags: Vec<Tag>) -> Way {
        Way {
            id,
            version,
            timestamp,
            changeset,
            uid,
            user,
            visible,
            refs,
            tags,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn changeset(&self) -> i64 {
        self.changeset
    }

    pub fn uid(&self) -> i32 {
        self.uid
    }

    pub fn user(&self) -> &String {
        &self.user
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn refs(&self) -> &Vec<i64> {
        &self.refs
    }
    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }
}