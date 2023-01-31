use crate::osm::model::node::Node;
use crate::osm::model::relation::Relation;
use crate::osm::model::way::Way;

#[derive(Debug)]
pub enum Element {
    Node {
        node: Node,
    },
    Way {
        way: Way,
    },
    Relation {
        relation: Relation,
    }
}