use crate::osm::apidb_dump::way_node_record::WayNodeRecord;
use crate::osm::apidb_dump::way_record::WayRecord;
use crate::osm::apidb_dump::way_tag_record::WayTagRecord;

#[derive(Debug)]
pub(crate) struct WayRelation{
    way: WayRecord,
    way_nodes: Vec<WayNodeRecord>,
    tags: Vec<WayTagRecord>,
}

impl WayRelation {
    pub(crate)  fn new(
        way: WayRecord,
        way_nodes: Vec<WayNodeRecord>,
        tags: Vec<WayTagRecord>,
    ) -> WayRelation {
        WayRelation {
            way,
            way_nodes,
            tags,
        }
    }

    pub(crate) fn way(&self) -> &WayRecord {
        &self.way
    }

    pub(crate) fn way_nodes(&self) -> &Vec<WayNodeRecord> {
        &self.way_nodes
    }

    pub(crate) fn take_way_nodes(&mut self) -> Vec<WayNodeRecord> {
        std::mem::take(&mut self.way_nodes)
    }

    pub(crate) fn tags(&self) -> &Vec<WayTagRecord> {
        &self.tags
    }

    pub(crate) fn take_tags(&mut self) -> Vec<WayTagRecord> {
        std::mem::take(&mut self.tags)
    }
}