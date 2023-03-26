use std::collections::HashMap;
use crate::osm::apidb_dump::table_def::TableDef;
use crate::osm::apidb_dump::table_reader::{TableIterator, TableReader};
use crate::osm::apidb_dump::table_record::TableRecord;
use crate::osm::apidb_dump::way_node_record::WayNodeRecord;
use crate::osm::apidb_dump::way_relation::WayRelation;
use crate::osm::apidb_dump::way_tag_record::WayTagRecord;

#[derive(Clone)]
pub(crate) struct WayRelationsReader {
    ways_reader: TableReader,
    way_nodes_reader: TableReader,
    way_tags_reader: TableReader,
}

impl WayRelationsReader {
    pub(crate) fn new(
        ways_def: &TableDef,
        way_nodes_def: &TableDef,
        way_tags_def: &TableDef,
    ) -> Result<WayRelationsReader, anyhow::Error> {
        let ways_reader = TableReader::new(ways_def)?;
        let way_nodes_reader = TableReader::new(way_nodes_def)?;
        let way_tags_reader = TableReader::new(way_tags_def)?;
        Ok(
            WayRelationsReader {
                ways_reader,
                way_nodes_reader,
                way_tags_reader,
            }
        )
    }
}

impl IntoIterator for WayRelationsReader {
    type Item = WayRelation;
    type IntoIter = WayRelationsIterator;

    fn into_iter(self) -> Self::IntoIter {
        WayRelationsIterator::new(&self).unwrap()
    }
}

pub(crate) struct WayRelationsIterator {
    reader: WayRelationsReader,
    ways_iterator: TableIterator,
    way_nodes_iterator: TableIterator,
    way_tags_iterator: TableIterator,
    next_way_node_record: Option<WayNodeRecord>,
    next_way_tag_record: Option<WayTagRecord>,
}


impl WayRelationsIterator {
    pub(crate) fn new(way_relations_reader: &WayRelationsReader) -> Result<WayRelationsIterator, anyhow::Error> {
        let reader = way_relations_reader.clone();
        let ways_iterator = reader.ways_reader.clone().into_iter();
        let way_nodes_iterator = reader.way_nodes_reader.clone().into_iter();
        let way_tags_iterator = reader.way_tags_reader.clone().into_iter();
        Ok(
            WayRelationsIterator {
                reader,
                ways_iterator,
                way_nodes_iterator,
                way_tags_iterator,
                next_way_node_record: None,
                next_way_tag_record: None,
            }
        )
    }
}

impl Iterator for WayRelationsIterator {
    type Item = WayRelation;

    fn next(&mut self) -> Option<Self::Item> {
        // ADD CONSTRAINT way_nodes_id_fkey FOREIGN KEY (way_id, version) REFERENCES public.ways(way_id, version);
        // ADD CONSTRAINT way_tags_id_fkey FOREIGN KEY (way_id, version) REFERENCES public.ways(way_id, version);

        if let Some(way) = self.ways_iterator.next() {
            if let TableRecord::Way { way_record } = way {
                let mut current_way_tags = Vec::<WayTagRecord>::new();
                if let Some(way_tag_record) = self.next_way_tag_record.take() {
                    if way_tag_record.way_id() == way_record.way_id() {
                        current_way_tags.push(way_tag_record);
                        while let Some(way_tag) = self.way_tags_iterator.next() {
                            if let TableRecord::WayTag { way_tag_record } = way_tag {
                                if way_tag_record.way_id() == way_record.way_id() && way_tag_record.version() == way_record.version() {
                                    current_way_tags.push(way_tag_record)
                                } else {
                                    self.next_way_tag_record = Some(way_tag_record);
                                    break;
                                }
                            } else {
                                panic!("Found incorrect record type, not a TableRecord:WayTag");
                            }
                        }
                    } else {
                        self.next_way_tag_record = Some(way_tag_record);
                    }
                } else {
                    for way_tag in self.way_tags_iterator.by_ref() {
                        if let TableRecord::WayTag { way_tag_record } = way_tag {
                            if way_tag_record.way_id() == way_record.way_id() && way_tag_record.version() == way_record.version() {
                                current_way_tags.push(way_tag_record)
                            } else {
                                self.next_way_tag_record = Some(way_tag_record);
                                break;
                            }
                        } else {
                            panic!("Found incorrect record type, not a TableRecord:WayTag");
                        }
                    }
                }

                let mut current_way_nodes = Vec::<WayNodeRecord>::new();
                if let Some(way_node_record) = self.next_way_node_record.take() {
                    if way_node_record.way_id() == way_record.way_id() && way_node_record.version() == way_record.version() {
                        current_way_nodes.push(way_node_record);
                        while let Some(way_node) = self.way_nodes_iterator.next() {
                            if let TableRecord::WayNode { way_node_record } = way_node {
                                if way_node_record.way_id() == way_record.way_id() && way_node_record.version() == way_record.version() {
                                    current_way_nodes.push(way_node_record)
                                } else {
                                    self.next_way_node_record = Some(way_node_record);
                                    break;
                                }
                            } else {
                                panic!("Found incorrect record type, not a TableRecord:WayNode");
                            }
                        }
                    } else {
                        self.next_way_node_record = Some(way_node_record);
                    }
                } else {
                    for way_node in self.way_nodes_iterator.by_ref() {
                        if let TableRecord::WayNode { way_node_record } = way_node {
                            if way_node_record.way_id() == way_record.way_id() && way_node_record.version() == way_record.version() {
                                current_way_nodes.push(way_node_record)
                            } else {
                                self.next_way_node_record = Some(way_node_record);
                                break;
                            }
                        } else {
                            panic!("Found incorrect record type, not a TableRecord:WayNode");
                        }
                    }
                }

                Some(
                    WayRelation::new(
                        way_record,
                        current_way_nodes,
                        current_way_tags,
                    )
                )
            } else {
                panic!("Found incorrect record type, not a TableRecord:Way");
            }
        } else {
            None
        }
    }
}
