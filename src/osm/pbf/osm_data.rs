use std::borrow::Borrow;
use std::io::Cursor;
use std::ops::Index;
use prost::Message;
use crate::error::GenericError;
use crate::{osm, osmpbf};

#[derive(Debug)]
pub struct OsmData {
    pub elements: Vec<osm::model::element::Element>,
    pub index: usize,
}

impl OsmData {
    pub fn new(index: usize, data: Vec<u8>) -> Result<OsmData, GenericError> {
        let primitive_block = osmpbf::PrimitiveBlock::decode(&mut Cursor::new(data))?;
        let string_table: Vec<String> = (&primitive_block.stringtable.s).into_iter()
            .map(
                |e| {
                    String::from_utf8(e.clone()).unwrap()
                }
            )
            .collect();
        let granularity = primitive_block.granularity() as i64;
        let date_granularity = primitive_block.date_granularity();
        let lat_offset = primitive_block.lat_offset();
        let lon_offset = primitive_block.lon_offset();
        let mut elements = Vec::<osm::model::element::Element>::new();
        for g in &primitive_block.primitivegroup {
            Self::read_dense(&g.dense, &string_table, granularity, date_granularity, lat_offset, lon_offset, &mut elements);
            Self::read_nodes(&g.nodes, &string_table, granularity, date_granularity, lat_offset, lon_offset, &mut elements);
            Self::read_ways(&g.ways, &string_table, granularity, date_granularity, &mut elements);
            Self::read_relations(&g.relations, &string_table, granularity, date_granularity, &mut elements);
            Self::read_changesets(&g.changesets, &string_table, granularity, date_granularity, lat_offset, lon_offset, &mut elements);
        }
        Ok(
            OsmData { index, elements }
        )
    }

    fn read_dense(dense_group: &Option<osmpbf::DenseNodes>, string_table: &Vec<String>, granularity: i64, date_granularity: i32, lat_offset: i64, lon_offset: i64, elements: &mut Vec<osm::model::element::Element>) {
        if let Some(dense) = dense_group {
            let mut last_id = 0_i64;
            let mut last_lat = 0_i64;
            let mut last_lon = 0_i64;

            let mut last_timestamp = 0_i64;
            let mut timestamp = 0_i64;
            let mut changeset = -1_i64;
            let mut uid = -1_i32;
            let mut last_user_sid = 0_i32;

            if let Some(info) = &dense.denseinfo {
                last_timestamp = 0_i64;
                changeset = 0_i64;
                uid = 0_i32;
                last_user_sid = 0_i32;
            }
            let mut visible = true;

            let mut key_val_iterator = <Vec<i32> as Borrow<Vec<i32>>>::borrow(&dense.keys_vals).into_iter();
            for (i, id) in <Vec<i64> as Borrow<Vec<i64>>>::borrow(&dense.id).into_iter().enumerate() {
                last_id = last_id + id;
                last_lat = last_lat + dense.lat[i];
                last_lon = last_lon + dense.lon[i];

                let mut user: String = String::default();
                if let Some(info) = &dense.denseinfo {
                    last_timestamp = last_timestamp + info.timestamp[i];
                    timestamp = last_timestamp * date_granularity as i64;
                    changeset = changeset + info.changeset[i];
                    uid = uid + info.uid[i];
                    last_user_sid = last_user_sid + info.user_sid[i];
                    let user_sid = last_user_sid as usize;
                    if info.visible.len() > i {
                        visible = info.visible[i];
                    } else {
                        visible = true;
                    }
                    user = string_table.index(user_sid).clone();
                }

                let mut tags = Vec::<osm::model::tag::Tag>::new();
                while let Some(key_val) = key_val_iterator.next() {
                    if *key_val == 0 {
                        break;
                    } else {
                        let key = *key_val as usize;
                        let val = *key_val_iterator.next().unwrap() as usize;
                        tags.push(
                            osm::model::tag::Tag::new(
                                string_table.index(key).clone(),
                                string_table.index(val).clone(),
                            )
                        );
                    }
                }

                let coordinate = osm::model::coordinate::Coordinate::new(
                    (lat_offset + (granularity * last_lat)) as f64 / 1000000000 as f64,
                    (lon_offset + (granularity * last_lon)) as f64 / 1000000000 as f64,
                );

                let node = osm::model::node::Node::new(
                    last_id,
                    coordinate,
                    last_timestamp,
                    changeset,
                    uid,
                    user,
                    visible,
                    tags,
                );
                elements.push(osm::model::element::Element::Node { node });
            }
        }
    }

    fn read_nodes(node_group: &Vec<osmpbf::Node>, string_table: &Vec<String>, granularity: i64, date_granularity: i32, lat_offset: i64, lon_offset: i64, elements: &mut Vec<osm::model::element::Element>) {
        for node in node_group {
            let id = node.id;
            let coordinate = osm::model::coordinate::Coordinate::new(
                (lat_offset + (granularity * node.lat)) as f64 / 1000000000 as f64,
                (lon_offset + (granularity * node.lon)) as f64 / 1000000000 as f64,
            );

            let (timestamp, changeset, uid, user, visible) =
                Self::read_info(string_table, date_granularity, &node.info);

            let mut tags = Vec::<osm::model::tag::Tag>::new();
            for i in 0..node.keys.len() {
                let k = string_table[node.keys[i] as usize].clone();
                let v = string_table[node.vals[i] as usize].clone();
                tags.push(osm::model::tag::Tag::new(k, v));
            }

            let node = osm::model::node::Node::new(
                id,
                coordinate,
                timestamp,
                changeset,
                uid,
                user,
                visible,
                tags,
            );
            elements.push(osm::model::element::Element::Node { node });
        }
    }

    fn read_info(string_table: &Vec<String>, date_granularity: i32, info_opt: &Option<osmpbf::Info>) -> (i64, i64, i32, String, bool) {
    let mut timestamp = -1_i64;
        let mut changeset = -1_i64;
        let mut uid = -1_i32;
        let mut user_sid = 0_usize;
        let mut user: String = String::default();
        let mut visible = true;

        if let Some(info) = info_opt {
            timestamp = info.timestamp.unwrap_or(0) * (date_granularity as i64);
            changeset = info.changeset.unwrap_or(-1);
            uid = info.uid.unwrap_or(-1);
            user_sid = info.user_sid.unwrap_or(0) as usize;
            visible = info.visible.unwrap_or(true);
            user = string_table[user_sid].clone();
        }
        (timestamp, changeset, uid, user, visible)
    }


    fn read_ways(way_group: &Vec<osmpbf::Way>, string_table: &Vec<String>, granularity: i64, date_granularity: i32, elements: &mut Vec<osm::model::element::Element>) {
        for way in way_group {
            let id = way.id;
            let (timestamp, changeset, uid, user, visible) =
                Self::read_info(string_table, date_granularity, &way.info);

            let mut refs = Vec::<i64>::new();
            let mut last_ref = 0_i64;
            for delta in &way.refs {
                last_ref = last_ref + delta;
                refs.push(last_ref);
            }

            let mut tags = Vec::<osm::model::tag::Tag>::new();
            for i in 0..way.keys.len() {
                let k = string_table[way.keys[i] as usize].clone();
                let v = string_table[way.vals[i] as usize].clone();
                tags.push(osm::model::tag::Tag::new(k, v));
            }

            let way = osm::model::way::Way::new(
                id,
                timestamp,
                changeset,
                uid,
                user,
                visible,
                refs,
                tags,
            );
            elements.push(osm::model::element::Element::Way { way });
        }
    }

    fn read_relations(relation_group: &Vec<osmpbf::Relation>, string_table: &Vec<String>, granularity: i64, date_granularity: i32, elements: &mut Vec<osm::model::element::Element>) {
        for relation in relation_group {
            let id = relation.id;
            let (timestamp, changeset, uid, user, visible) =
                Self::read_info(string_table, date_granularity, &relation.info);

            let mut members = Vec::<osm::model::relation::Member>::new();
            let mut last_memid = 0_i64;
            for i in 0..relation.memids.len() {
                last_memid = last_memid + relation.memids[i];
                let role = string_table[relation.roles_sid[i] as usize].clone();
                let member = osm::model::relation::MemberData::new(last_memid, role);
                if let Some(member_type) = osmpbf::relation::MemberType::from_i32(relation.types[i]) {
                   match member_type {
                       osmpbf::relation::MemberType::Node => {
                           members.push(osm::model::relation::Member::Node {member});
                       }
                       osmpbf::relation::MemberType::Way => {
                           members.push(osm::model::relation::Member::Way {member});
                       }
                       osmpbf::relation::MemberType::Relation => {
                           members.push(osm::model::relation::Member::Relation {member});
                       }
                   }
                } else {
                    // unlikely
                    panic!("Non existing relation member type: {}", relation.types[i]);
                }

            }

            let mut tags = Vec::<osm::model::tag::Tag>::new();
            for i in 0..relation.keys.len() {
                let k = string_table[relation.keys[i] as usize].clone();
                let v = string_table[relation.vals[i] as usize].clone();
                tags.push(osm::model::tag::Tag::new(k, v));
            }

            let relation = osm::model::relation::Relation::new(
                id,
                timestamp,
                changeset,
                uid,
                user,
                visible,
                members,
                tags,
            );
            elements.push(osm::model::element::Element::Relation { relation });
        }
    }

    fn read_changesets(changeset_group: &Vec<osmpbf::ChangeSet>, string_table: &Vec<String>, granularity: i64, date_granularity: i32, lat_offset: i64, lon_offset: i64, elements: &mut Vec<osm::model::element::Element>) {
        for changeset in changeset_group {
            panic!("According to documentation changesets are not used");
        }
    }
}
