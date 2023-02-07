use crate::osm::model::coordinate::Coordinate;
use crate::osm::model::tag::Tag;

#[derive(Debug)]
pub struct Node {
    id: i64,
    coordinate: Coordinate,
    timestamp: i64,
    changeset: i64,
    uid: i32,
    user: String,
    visible: bool,
    tags: Vec<Tag>,
}

impl Node {
    pub fn new(id: i64, coordinate: Coordinate, timestamp: i64, changeset: i64, uid: i32, user: String, visible: bool, tags: Vec<Tag>) -> Node {
        Node {
            id,
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
}