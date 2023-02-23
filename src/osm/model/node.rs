use crate::osm::model::coordinate::Coordinate;
use crate::osm::model::tag::Tag;

#[derive(Debug)]
pub struct Node {
    id: i64,
    version: i32,
    coordinate: Coordinate,
    timestamp: i64,
    changeset: i64,
    uid: i32,
    user: String,
    visible: bool,
    tags: Vec<Tag>,
}

impl Node {
    pub fn new(id: i64, version: i32, coordinate: Coordinate, timestamp: i64, changeset: i64, uid: i32, user: String, visible: bool, tags: Vec<Tag>) -> Node {
        Node {
            id,
            version,
            coordinate,
            timestamp,
            changeset,
            uid,
            user,
            visible,
            tags,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn coordinate(&self) -> &Coordinate {
        &self.coordinate
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
        self.visible.clone()
    }

    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }
}