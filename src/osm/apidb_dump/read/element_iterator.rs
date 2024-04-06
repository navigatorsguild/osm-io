use std::collections::HashMap;

use anyhow::anyhow;
use transient_btree_index::{BtreeConfig, BtreeIndex};

use crate::osm::apidb_dump::read::node_relations_reader::{NodeRelationsIterator, NodeRelationsReader};
use crate::osm::apidb_dump::read::relation_member_record::RelationMemberType;
use crate::osm::apidb_dump::read::relation_relations_reader::{RelationRelationsIterator, RelationRelationsReader};
use crate::osm::apidb_dump::read::table_def::TableDef;
use crate::osm::apidb_dump::read::table_reader::TableReader;
use crate::osm::apidb_dump::read::table_record::TableRecord;
use crate::osm::apidb_dump::read::way_relations_reader::{WayRelationsIterator, WayRelationsReader};
use crate::osm::model::coordinate::Coordinate;
use crate::osm::model::element::Element;
use crate::osm::model::node::Node;
use crate::osm::model::relation;
use crate::osm::model::relation::Relation;
use crate::osm::model::tag::Tag;
use crate::osm::model::way::Way;

enum IterationState {
    Start,
    Nodes,
    Ways,
    Relations,
    End,
}

pub struct ElementIterator {
    user_index: BtreeIndex<i64, String>,
    changeset_user_index: BtreeIndex<i64, i64>,
    iteration_state: IterationState,
    node_relations_iterator: NodeRelationsIterator,
    way_relations_iterator: WayRelationsIterator,
    relation_relations_iterator: RelationRelationsIterator,
}

impl ElementIterator {
    pub(crate) fn new(tables: HashMap<String, TableDef>) -> Result<ElementIterator, anyhow::Error> {
        let user_index = Self::index_users(&tables)?;
        let changeset_user_index = Self::index_changesets(&tables)?;
        let node_relations_reader = NodeRelationsReader::new(
            tables.get("public.nodes").ok_or(anyhow!("missing table definition for public.nodes"))?,
            tables.get("public.node_tags").ok_or(anyhow!("missing table definition for public.node_tags"))?,
        )?;
        let node_relations_iterator = node_relations_reader.into_iter();

        let way_relations_reader = WayRelationsReader::new(
            tables.get("public.ways").ok_or(anyhow!("missing table definition for public.ways"))?,
            tables.get("public.way_nodes").ok_or(anyhow!("missing table definition for public.way_nodes"))?,
            tables.get("public.way_tags").ok_or(anyhow!("missing table definition for public.way_tags"))?,
        )?;
        let way_relations_iterator = way_relations_reader.into_iter();

        let relation_relations_reader = RelationRelationsReader::new(
            tables.get("public.relations").ok_or(anyhow!("missing table definition for public.relations"))?,
            tables.get("public.relation_members").ok_or(anyhow!("missing table definition for public.relation_members"))?,
            tables.get("public.relation_tags").ok_or(anyhow!("missing table definition for public.relation_tags"))?,
        )?;
        let relation_relations_iterator = relation_relations_reader.into_iter();

        Ok(
            ElementIterator {
                user_index,
                changeset_user_index,
                iteration_state: IterationState::Start,
                node_relations_iterator,
                way_relations_iterator,
                relation_relations_iterator,
            }
        )
    }

    fn index_changesets(tables: &HashMap<String, TableDef>) -> Result<BtreeIndex<i64, i64>, anyhow::Error> {
        let mut changeset_user_index = BtreeIndex::<i64, i64>::with_capacity(BtreeConfig::default(), 0)?;
        let reader = TableReader::new(tables.get("public.changesets").unwrap())?;
        for record in reader {
            if let TableRecord::Changeset { changeset_record } = record {
                changeset_user_index.insert(changeset_record.id(), changeset_record.user_id()).unwrap();
            } else {
                return Err(anyhow!("Not a changeset record"));
            }
        }
        Ok(changeset_user_index)
    }

    fn index_users(tables: &HashMap<String, TableDef>) -> Result<BtreeIndex<i64, String>, anyhow::Error> {
        let mut user_index = BtreeIndex::<i64, String>::with_capacity(BtreeConfig::default(), 0)?;
        let reader = TableReader::new(tables.get("public.users").unwrap())?;
        for record in reader {
            if let TableRecord::User { user_record } = &record {
                user_index.insert(user_record.id(), user_record.display_name().clone()).unwrap();
            } else {
                return Err(anyhow!("Not a user record"));
            }
        }
        Ok(user_index)
    }
}

impl Iterator for ElementIterator {
    type Item = Element;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iteration_state {
            IterationState::Start => {
                log::info!("Start reading Nodes");
                self.iteration_state = IterationState::Nodes;
                self.next()
            }
            IterationState::Nodes => {
                let node_relation = self.node_relations_iterator.next();
                match node_relation {
                    None => {
                        log::info!("Start reading Ways");
                        self.iteration_state = IterationState::Ways;
                        Some(Element::Sentinel)
                    }
                    Some(mut n) => {
                        let changeset_id = n.node().changeset_id();
                        let uid = self.changeset_user_index.get(&changeset_id).unwrap().unwrap();
                        let user = self.user_index.get(&uid).unwrap().unwrap();
                        Some(
                            Element::Node {
                                node: Node::new(
                                    n.node().node_id(),
                                    n.node().version() as i32,
                                    Coordinate::new(
                                        n.node().latitude() as f64 / 10000000.0f64,
                                        n.node().longitude() as f64 / 10000000.0f64,
                                    ),
                                    n.node().timestamp().and_utc().timestamp(),
                                    changeset_id,
                                    uid as i32,
                                    user,
                                    n.node().visible(),
                                    n.take_tags().into_iter().map(
                                        |mut tag_record| {
                                            Tag::new(tag_record.take_k(), tag_record.take_v())
                                        }
                                    ).collect(),
                                )
                            }
                        )
                    }
                }
            }
            IterationState::Ways => {
                let way_relation = self.way_relations_iterator.next();
                match way_relation {
                    None => {
                        log::info!("Start reading Relations");
                        self.iteration_state = IterationState::Relations;
                        Some(Element::Sentinel)
                    }
                    Some(mut w) => {
                        let changeset_id = w.way().changeset_id();
                        let uid = self.changeset_user_index.get(&changeset_id).unwrap().unwrap();
                        let user = self.user_index.get(&uid).unwrap().unwrap();
                        Some(
                            Element::Way {
                                way: Way::new(
                                    w.way().way_id(),
                                    w.way().version() as i32,
                                    w.way().timestamp().and_utc().timestamp(),
                                    changeset_id,
                                    uid as i32,
                                    user,
                                    w.way().visible(),
                                    w.take_way_nodes().into_iter().map(
                                        |way_node_record| {
                                            way_node_record.node_id()
                                        }
                                    ).collect(),
                                    w.take_tags().into_iter().map(
                                        |mut tag_record| {
                                            Tag::new(tag_record.take_k(), tag_record.take_v())
                                        }
                                    ).collect(),
                                )
                            }
                        )
                    }
                }
            }
            IterationState::Relations => {
                let relation_relation = self.relation_relations_iterator.next();
                match relation_relation {
                    None => {
                        self.iteration_state = IterationState::End;
                        Some(Element::Sentinel)
                    }
                    Some(mut r) => {
                        let changeset_id = r.relation().changeset_id();
                        let uid = self.changeset_user_index.get(&changeset_id).unwrap().unwrap();
                        let user = self.user_index.get(&uid).unwrap().unwrap();
                        Some(
                            Element::Relation {
                                relation: Relation::new(
                                    r.relation().relation_id(),
                                    r.relation().version() as i32,
                                    r.relation().timestamp().and_utc().timestamp(),
                                    changeset_id,
                                    uid as i32,
                                    user,
                                    r.relation().visible(),
                                    r.take_relation_members().into_iter().map(
                                        |mut relation_member_record| {
                                            let relation_member_data = relation::MemberData::new(
                                                relation_member_record.member_id(),
                                                relation_member_record.take_member_role(),
                                            );
                                            match relation_member_record.member_type() {
                                                RelationMemberType::Node => {
                                                    relation::Member::Node { member: relation_member_data }
                                                }
                                                RelationMemberType::Way => {
                                                    relation::Member::Way { member: relation_member_data }
                                                }
                                                RelationMemberType::Relation => {
                                                    relation::Member::Relation { member: relation_member_data }
                                                }
                                            }
                                        }
                                    ).collect(),
                                    r.take_tags().into_iter().map(
                                        |mut tag_record| {
                                            Tag::new(tag_record.take_k(), tag_record.take_v())
                                        }
                                    ).collect(),
                                )
                            }
                        )
                    }
                }
            }
            IterationState::End => {
                log::info!("Complete iteration");
                None
            }
        }
    }
}
