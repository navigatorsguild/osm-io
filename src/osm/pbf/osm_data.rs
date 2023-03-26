use std::borrow::Borrow;
use std::io::Cursor;
use std::ops::Index;
use prost::Message;
use crate::{osm, osmpbf};
use crate::osm::model::bounding_box::BoundingBox;
use crate::osm::model::element::Element;
use crate::osm::pbf::dense_group_builder::DenseGroupBuilder;
use crate::osm::pbf::relations_group_builder::RelationsGroupBuilder;
use crate::osm::pbf::string_table_builder::StringTableBuilder;
use crate::osm::pbf::ways_group_builder::WaysGroupBuilder;
use crate::osmpbf::{PrimitiveBlock, PrimitiveGroup};

#[derive(Debug, Default)]
pub struct OsmData {
    // TODO: check impact on performance if changed to deq
    elements: Vec<Element>,
    pub index: usize,
    pub bounding_box: Option<BoundingBox>,
    // TODO: revisit pub members.
}

impl OsmData {
    pub fn new(index: usize, data: Vec<u8>) -> Result<OsmData, anyhow::Error> {
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
        let mut elements = Vec::<Element>::new();
        for g in &primitive_block.primitivegroup {
            Self::read_dense(&g.dense, &string_table, granularity, date_granularity, lat_offset, lon_offset, &mut elements);
            Self::read_nodes(&g.nodes, &string_table, granularity, date_granularity, lat_offset, lon_offset, &mut elements);
            Self::read_ways(&g.ways, &string_table, granularity, date_granularity, &mut elements);
            Self::read_relations(&g.relations, &string_table, granularity, date_granularity, &mut elements);
            Self::read_changesets(&g.changesets, &string_table, granularity, date_granularity, lat_offset, lon_offset, &mut elements);
        }
        Ok(
            OsmData { index, elements, bounding_box: None }
        )
    }

    pub fn from_elements(index: usize, elements: Vec<Element>, bounding_box: Option<BoundingBox>) -> OsmData {
        OsmData { index, elements, bounding_box }
    }

    pub fn compute_bounding_box(&self) -> Option<BoundingBox> {
        if self.bounding_box.is_none() {
            self.recompute_bounding_box()
        } else {
            self.bounding_box.clone()
        }
    }

    pub fn recompute_bounding_box(&self) -> Option<BoundingBox> {
        let mut result = None;
        for element in &self.elements {
            match element {
                Element::Node { node } => {
                    if result.is_none() {
                        result = Some(
                            BoundingBox::new(
                                node.coordinate().lon(),
                                node.coordinate().lon(),
                                node.coordinate().lat(),
                                node.coordinate().lat(),
                            )
                        )
                    } else {
                        result.as_mut().unwrap().merge_point(node.coordinate());
                    }
                }
                Element::Way { .. } => { break; }
                Element::Relation { .. } => { break; }
                Element::Sentinel => { break; }
            }
        }
        result
    }

    fn read_dense(dense_group: &Option<osmpbf::DenseNodes>, string_table: &Vec<String>, granularity: i64, date_granularity: i32, lat_offset: i64, lon_offset: i64, elements: &mut Vec<Element>) {
        if let Some(dense) = dense_group {
            let mut last_id = 0_i64;
            let mut last_lat = 0_i64;
            let mut last_lon = 0_i64;

            let mut last_timestamp = 0_i64;
            let mut timestamp = 0_i64;
            let mut last_changeset = -1_i64;
            let mut uid = -1_i32;
            let mut last_user_sid = 0_i32;

            if let Some(_info) = &dense.denseinfo {
                last_timestamp = 0_i64;
                last_changeset = 0_i64;
                uid = 0_i32;
                last_user_sid = 0_i32;
            }
            let mut visible = true;

            let mut key_val_iterator = <Vec<i32> as Borrow<Vec<i32>>>::borrow(&dense.keys_vals).into_iter();
            for (i, id) in <Vec<i64> as Borrow<Vec<i64>>>::borrow(&dense.id).into_iter().enumerate() {
                last_id = last_id + id;
                last_lat = last_lat + dense.lat[i];
                last_lon = last_lon + dense.lon[i];

                let mut version = 0_i32;
                let mut user: String = String::default();
                if let Some(info) = &dense.denseinfo {
                    last_timestamp = last_timestamp + info.timestamp[i];
                    timestamp = last_timestamp * date_granularity as i64;
                    last_changeset = last_changeset + info.changeset[i];
                    uid = uid + info.uid[i];
                    last_user_sid = last_user_sid + info.user_sid[i];
                    let user_sid = last_user_sid as usize;
                    if info.visible.len() > i {
                        visible = info.visible[i];
                    } else {
                        visible = true;
                    }
                    version = info.version[i];
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
                    version,
                    coordinate,
                    timestamp,
                    last_changeset,
                    uid,
                    user,
                    visible,
                    tags,
                );
                elements.push(Element::Node { node });
            }
        }
    }

    fn read_nodes(node_group: &Vec<osmpbf::Node>, string_table: &Vec<String>, granularity: i64, date_granularity: i32, lat_offset: i64, lon_offset: i64, elements: &mut Vec<Element>) {
        for node in node_group {
            let id = node.id;
            let coordinate = osm::model::coordinate::Coordinate::new(
                (lat_offset + (granularity * node.lat)) as f64 / 1000000000 as f64,
                (lon_offset + (granularity * node.lon)) as f64 / 1000000000 as f64,
            );

            let (timestamp, changeset, uid, user, visible, version) =
                Self::read_info(string_table, date_granularity, &node.info);

            let mut tags = Vec::<osm::model::tag::Tag>::new();
            for i in 0..node.keys.len() {
                let k = string_table[node.keys[i] as usize].clone();
                let v = string_table[node.vals[i] as usize].clone();
                tags.push(osm::model::tag::Tag::new(k, v));
            }

            let node = osm::model::node::Node::new(
                id,
                version,
                coordinate,
                timestamp,
                changeset,
                uid,
                user,
                visible,
                tags,
            );
            elements.push(Element::Node { node });
        }
    }

    fn read_info(string_table: &Vec<String>, date_granularity: i32, info_opt: &Option<osmpbf::Info>) -> (i64, i64, i32, String, bool, i32) {
        let mut timestamp = -1_i64;
        let mut changeset = -1_i64;
        let mut uid = -1_i32;
        let user_sid;
        let mut user: String = String::default();
        let mut visible = true;
        let mut version = 0_i32;

        if let Some(info) = info_opt {
            timestamp = info.timestamp.unwrap_or(0) * (date_granularity as i64);
            changeset = info.changeset.unwrap_or(-1);
            uid = info.uid.unwrap_or(-1);
            user_sid = info.user_sid.unwrap_or(0) as usize;
            visible = info.visible.unwrap_or(true);
            user = string_table[user_sid].clone();
            version = info.version();
        }
        (timestamp, changeset, uid, user, visible, version)
    }


    fn read_ways(way_group: &Vec<osmpbf::Way>, string_table: &Vec<String>, _granularity: i64, date_granularity: i32, elements: &mut Vec<Element>) {
        for way in way_group {
            let id = way.id;
            let (timestamp, changeset, uid, user, visible, version) =
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
                version,
                timestamp,
                changeset,
                uid,
                user,
                visible,
                refs,
                tags,
            );
            elements.push(Element::Way { way });
        }
    }

    fn read_relations(relation_group: &Vec<osmpbf::Relation>, string_table: &Vec<String>, _granularity: i64, date_granularity: i32, elements: &mut Vec<Element>) {
        for relation in relation_group {
            let id = relation.id;
            let (timestamp, changeset, uid, user, visible, version) =
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
                            members.push(osm::model::relation::Member::Node { member });
                        }
                        osmpbf::relation::MemberType::Way => {
                            members.push(osm::model::relation::Member::Way { member });
                        }
                        osmpbf::relation::MemberType::Relation => {
                            members.push(osm::model::relation::Member::Relation { member });
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
                version,
                timestamp,
                changeset,
                uid,
                user,
                visible,
                members,
                tags,
            );
            elements.push(Element::Relation { relation });
        }
    }

    fn read_changesets(changeset_group: &Vec<osmpbf::ChangeSet>, _string_table: &Vec<String>, _granularity: i64, _date_granularity: i32, _lat_offset: i64, _lon_offset: i64, _elements: &mut Vec<Element>) {
        for _changeset in changeset_group {
            panic!("According to documentation changesets are not used");
        }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, anyhow::Error> {
        let mut string_table_builder = StringTableBuilder::new();
        let granularity = 100_i32;
        let date_granularity = 1000_i32;
        let lat_offset = 0_i64;
        let lon_offset = 0_i64;

        let mut dense_group_builder = None;
        let mut ways_group_builder = None;
        let mut relations_group_builder = None;
        for element in &self.elements {
            match element {
                Element::Node { node } => {
                    if dense_group_builder.is_none() {
                        dense_group_builder = Some(
                            DenseGroupBuilder::new(granularity, date_granularity, lat_offset, lon_offset, node, &mut string_table_builder)
                        );
                    } else {
                        dense_group_builder.as_mut().unwrap().add(node, &mut string_table_builder)
                    }
                }
                Element::Way { way } => {
                    if ways_group_builder.is_none() {
                        ways_group_builder = Some(
                            WaysGroupBuilder::new(date_granularity, way, &mut string_table_builder)
                        );
                    } else {
                        ways_group_builder.as_mut().unwrap().add(way, &mut string_table_builder)
                    }
                }
                Element::Relation { relation } => {
                    if relations_group_builder.is_none() {
                        relations_group_builder = Some(
                            RelationsGroupBuilder::new(date_granularity, relation, &mut string_table_builder)
                        );
                    } else {
                        relations_group_builder.as_mut().unwrap().add(relation, &mut string_table_builder)
                    }
                }
                Element::Sentinel => {}
            }
        }
        let mut primitivegroup = PrimitiveGroup::default();
        if dense_group_builder.is_some() {
            primitivegroup = dense_group_builder.unwrap().build();
        } else if ways_group_builder.is_some() {
            primitivegroup = ways_group_builder.unwrap().build();
        } else if relations_group_builder.is_some() {
            primitivegroup = relations_group_builder.unwrap().build();
        }

        let stringtable = string_table_builder.build();

        let primitive_block = PrimitiveBlock {
            stringtable,
            primitivegroup: vec![primitivegroup],
            granularity: Some(granularity),
            lat_offset: Some(lat_offset),
            lon_offset: Some(lon_offset),
            date_granularity: Some(date_granularity),
        };

        let mut buf = Vec::<u8>::with_capacity(primitive_block.encoded_len());
        primitive_block.encode(&mut buf)?;
        Ok(buf)
    }

    pub fn elements(&self) -> &Vec<Element> {
        &self.elements
    }

    pub fn take_elements(&mut self) -> Vec<Element> {
        // std::mem::replace(&mut self.elements, Vec::<Element>::default())
        std::mem::take(&mut self.elements)
    }
}
