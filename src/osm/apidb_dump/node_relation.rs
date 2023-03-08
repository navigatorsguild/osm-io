use crate::osm::apidb_dump::node_record::NodeRecord;
use crate::osm::apidb_dump::node_tag_record::NodeTagRecord;
use crate::osm::model::node::Node;

#[derive(Debug)]
pub(crate) struct NodeRelation{
    node: NodeRecord,
    tags: Vec<NodeTagRecord>,
}

impl NodeRelation {
    pub(crate)  fn new(
        node: NodeRecord,
        tags: Vec<NodeTagRecord>,
    ) -> NodeRelation {
        NodeRelation {
            node,
            tags,
        }
    }

    pub(crate) fn node(&self) -> &NodeRecord {
        &self.node
    }

    pub(crate) fn tags(&self) -> &Vec<NodeTagRecord> {
        &self.tags
    }

    pub(crate) fn take_tags(&mut self) -> Vec<NodeTagRecord> {
        std::mem::take(&mut self.tags)
    }
}