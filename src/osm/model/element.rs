use std::cmp::Ordering;

use crate::osm::model::node::Node;
use crate::osm::model::relation::Relation;
use crate::osm::model::way::Way;

#[derive(Debug, Clone)]
pub enum Element {
    Node {
        node: Node,
    },
    Way {
        way: Way,
    },
    Relation {
        relation: Relation,
    },
    Sentinel,
}

impl Element {
    pub fn same_type(e1: &Element, e2: &Element) -> bool {
        match e1 {
            Element::Node { .. } => {
                match e2 {
                    Element::Node { .. } => { true }
                    Element::Way { .. } => { false }
                    Element::Relation { .. } => { false }
                    Element::Sentinel => { false }
                }
            }
            Element::Way { .. } => {
                match e2 {
                    Element::Node { .. } => { false }
                    Element::Way { .. } => { true }
                    Element::Relation { .. } => { false }
                    Element::Sentinel => { false }
                }
            }
            Element::Relation { .. } => {
                match e2 {
                    Element::Node { .. } => { false }
                    Element::Way { .. } => { false }
                    Element::Relation { .. } => { true }
                    Element::Sentinel => { false }
                }
            }
            Element::Sentinel => {
                match e2 {
                    Element::Node { .. } => { false }
                    Element::Way { .. } => { false }
                    Element::Relation { .. } => { false }
                    Element::Sentinel => { true }
                }
            }
        }
    }

    pub fn is_node(&self) -> bool {
        match self {
            Element::Node { .. } => {
                true
            }
            Element::Way { .. } => {
                false
            }
            Element::Relation { .. } => {
                false
            }
            Element::Sentinel => {
                false
            }
        }
    }

    pub fn is_way(&self) -> bool {
        match self {
            Element::Node { .. } => {
                false
            }
            Element::Way { .. } => {
                true
            }
            Element::Relation { .. } => {
                false
            }
            Element::Sentinel => {
                false
            }
        }
    }

    pub fn is_relation(&self) -> bool {
        match self {
            Element::Node { .. } => {
                false
            }
            Element::Way { .. } => {
                false
            }
            Element::Relation { .. } => {
                true
            }
            Element::Sentinel => {
                false
            }
        }
    }

    pub fn is_sentinel(&self) -> bool {
        match self {
            Element::Node { .. } => {
                false
            }
            Element::Way { .. } => {
                false
            }
            Element::Relation { .. } => {
                false
            }
            Element::Sentinel => {
                true
            }
        }
    }
}

impl Eq for Element {}

impl PartialEq<Self> for Element {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Element::Node { node } => {
                let id = node.id();
                let version = node.version();
                match other {
                    Element::Node { node } => {
                        id.eq(&node.id()) && version.eq(&node.version())
                    }
                    Element::Way { .. } => {
                        false
                    }
                    Element::Relation { .. } => {
                        false
                    }
                    Element::Sentinel => {
                        false
                    }
                }
            }
            Element::Way { way } => {
                let id = way.id();
                let version = way.version();
                match other {
                    Element::Node { .. } => {
                        false
                    }
                    Element::Way { way } => {
                        id.eq(&way.id()) && version.eq(&way.version())
                    }
                    Element::Relation { .. } => {
                        false
                    }
                    Element::Sentinel => {
                        false
                    }
                }
            }
            Element::Relation { relation } => {
                let id = relation.id();
                let version = relation.version();
                match other {
                    Element::Node { .. } => {
                        false
                    }
                    Element::Way { .. } => {
                        false
                    }
                    Element::Relation { relation } => {
                        id.eq(&relation.id()) && version.eq(&relation.version())
                    }
                    Element::Sentinel => {
                        false
                    }
                }
            }
            Element::Sentinel => {
                match other {
                    Element::Node { .. } => {
                        false
                    }
                    Element::Way { .. } => {
                        false
                    }
                    Element::Relation { .. } => {
                        false
                    }
                    Element::Sentinel => {
                        true
                    }
                }
            }
        }
    }
}

impl PartialOrd<Self> for Element {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Element {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Element::Node { node } => {
                let id = node.id();
                let version = node.version();
                match other {
                    Element::Node { node } => {
                        match id.cmp(&node.id()) {
                            Ordering::Less => {
                                Ordering::Less
                            }
                            Ordering::Equal => {
                                version.cmp(&node.version())
                            }
                            Ordering::Greater => {
                                Ordering::Greater
                            }
                        }
                    }
                    Element::Way { .. } => {
                        Ordering::Less
                    }
                    Element::Relation { .. } => {
                        Ordering::Less
                    }
                    Element::Sentinel => {
                        Ordering::Less
                    }
                }
            }
            Element::Way { way } => {
                let id = way.id();
                let version = way.version();
                match other {
                    Element::Node { .. } => {
                        Ordering::Greater
                    }
                    Element::Way { way } => {
                        match id.cmp(&way.id()) {
                            Ordering::Less => {
                                Ordering::Less
                            }
                            Ordering::Equal => {
                                version.cmp(&way.version())
                            }
                            Ordering::Greater => {
                                Ordering::Greater
                            }
                        }
                    }
                    Element::Relation { .. } => {
                        Ordering::Less
                    }
                    Element::Sentinel => {
                        Ordering::Less
                    }
                }
            }
            Element::Relation { relation } => {
                let id = relation.id();
                let version = relation.version();
                match other {
                    Element::Node { .. } => {
                        Ordering::Greater
                    }
                    Element::Way { .. } => {
                        Ordering::Greater
                    }
                    Element::Relation { relation } => {
                        match id.cmp(&relation.id()) {
                            Ordering::Less => {
                                Ordering::Less
                            }
                            Ordering::Equal => {
                                version.cmp(&relation.version())
                            }
                            Ordering::Greater => {
                                Ordering::Greater
                            }
                        }
                    }
                    Element::Sentinel => {
                        Ordering::Less
                    }
                }
            }
            Element::Sentinel => {
                match other {
                    Element::Node { .. } => {
                        Ordering::Less
                    }
                    Element::Way { .. } => {
                        Ordering::Less
                    }
                    Element::Relation { .. } => {
                        Ordering::Less
                    }
                    Element::Sentinel => {
                        Ordering::Equal
                    }
                }
            }
        }
    }
}