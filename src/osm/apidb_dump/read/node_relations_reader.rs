use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use crate::osm::apidb_dump::read::node_record::NodeRecord;
use crate::osm::apidb_dump::read::node_relation::NodeRelation;
use crate::osm::apidb_dump::read::node_tag_record::NodeTagRecord;
use crate::osm::apidb_dump::read::table_def::TableDef;
use crate::osm::apidb_dump::read::table_reader::{TableIterator, TableReader};
use crate::osm::apidb_dump::read::table_record::TableRecord;

#[derive(Clone)]
pub(crate) struct NodeRelationsReader {
    nodes_reader: TableReader,
    node_tags_reader: TableReader,
}

impl NodeRelationsReader {
    pub(crate) fn new(nodes_def: &TableDef, node_tags_def: &TableDef) -> Result<Self, anyhow::Error> {
        let nodes_reader = TableReader::new(nodes_def)?;
        let node_tags_reader = TableReader::new(node_tags_def)?;
        Ok(
            NodeRelationsReader {
                nodes_reader,
                node_tags_reader,
            }
        )
    }
}

impl IntoIterator for NodeRelationsReader {
    type Item = NodeRelation;
    type IntoIter = NodeRelationsIterator;

    fn into_iter(self) -> Self::IntoIter {
        NodeRelationsIterator::new(&self).unwrap()
    }
}

pub(crate) struct NodeRelationsIterator {
    reader: NodeRelationsReader,
    nodes_iterator: TableIterator,
    node_tags_iterator: TableIterator,
    next_node_tag_record: Option<NodeTagRecord>,
}

impl NodeRelationsIterator {
    pub(crate) fn new(node_relations_reader: &NodeRelationsReader) -> Result<NodeRelationsIterator, anyhow::Error> {
        let reader = node_relations_reader.clone();
        let nodes_iterator = reader.nodes_reader.clone().into_iter();
        let node_tags_iterator = reader.node_tags_reader.clone().into_iter();
        Ok(
            NodeRelationsIterator {
                reader,
                nodes_iterator,
                node_tags_iterator,
                next_node_tag_record: None,
            }
        )
    }
}

impl Iterator for NodeRelationsIterator {
    type Item = NodeRelation;

    fn next(&mut self) -> Option<Self::Item> {
        // ADD CONSTRAINT node_tags_id_fkey FOREIGN KEY (node_id, version) REFERENCES public.nodes(node_id, version);
        if let Some(node) = self.nodes_iterator.next() {
            if let TableRecord::Node { node_record } = node {
                let mut current_node_tags = Vec::<NodeTagRecord>::new();
                if let Some(node_tag_record) = self.next_node_tag_record.take() {
                    if node_tag_record.node_id() == node_record.node_id() && node_tag_record.version() == node_record.version() {
                        current_node_tags.push(node_tag_record);
                        while let Some(node_tag) = self.node_tags_iterator.next() {
                            if let TableRecord::NodeTag { node_tag_record } = node_tag {
                                if node_tag_record.node_id() == node_record.node_id() && node_tag_record.version() == node_record.version() {
                                    current_node_tags.push(node_tag_record)
                                } else {
                                    self.next_node_tag_record = Some(node_tag_record);
                                    break;
                                }
                            } else {
                                panic!("Found incorrect record type, not a TableRecord:NodeTag");
                            }
                        }
                    } else {
                        self.next_node_tag_record = Some(node_tag_record);
                    }
                } else {
                    for node_tag in self.node_tags_iterator.by_ref() {
                        if let TableRecord::NodeTag { node_tag_record } = node_tag {
                            if node_tag_record.node_id() == node_record.node_id() && node_tag_record.version() == node_record.version() {
                                current_node_tags.push(node_tag_record)
                            } else {
                                self.next_node_tag_record = Some(node_tag_record);
                                break;
                            }
                        } else {
                            panic!("Found incorrect record type, not a TableRecord:NodeTag");
                        }
                    }
                }

                Some(
                    NodeRelation::new(
                        node_record,
                        current_node_tags,
                    )
                )
            } else {
                panic!("Found incorrect record type, not a TableRecord:Node");
                None
            }
        } else {
            None
        }
    }
}