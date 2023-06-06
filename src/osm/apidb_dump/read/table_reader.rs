use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::str::FromStr;

use anyhow::anyhow;

use crate::osm::apidb_dump::read::changeset_record::ChangesetRecord;
use crate::osm::apidb_dump::read::node_record::NodeRecord;
use crate::osm::apidb_dump::read::node_tag_record::NodeTagRecord;
use crate::osm::apidb_dump::read::relation_member_record::{RelationMemberRecord, RelationMemberType};
use crate::osm::apidb_dump::read::relation_record::RelationRecord;
use crate::osm::apidb_dump::read::relation_tag_record::RelationTagRecord;
use crate::osm::apidb_dump::read::table_def::TableDef;
use crate::osm::apidb_dump::read::table_fields::TableFields;
use crate::osm::apidb_dump::read::table_record::TableRecord;
use crate::osm::apidb_dump::read::user_record::{FormatEnum, UserRecord, UserStatus};
use crate::osm::apidb_dump::read::way_node_record::WayNodeRecord;
use crate::osm::apidb_dump::read::way_record::WayRecord;
use crate::osm::apidb_dump::read::way_tag_record::WayTagRecord;
use crate::osm::apidb_dump::sql::{parse_sql_bool, parse_sql_null_string, parse_sql_time};

struct RecordBuilder {
    f: fn(&String, &TableDef, usize) -> Option<TableRecord>,
    table_def: TableDef,
}

impl RecordBuilder {
    fn build(&self, line: &String, line_number: usize) -> Option<TableRecord> {
        (self.f)(line, &self.table_def, line_number)
    }
}

#[derive(Clone)]
pub(crate) struct TableReader {
    table_def: TableDef,
}

impl TableReader {
    pub(crate) fn new(table_def: &TableDef) -> Result<TableReader, anyhow::Error> {
        Ok(
            TableReader {
                table_def: table_def.clone(),
            }
        )
    }

    fn create_record_builder(&self) -> Result<RecordBuilder, anyhow::Error> {
        match self.table_def.name().as_str() {
            "public.nodes" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_node,
                        table_def: self.table_def.clone(),
                    }
                )
            }
            "public.node_tags" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_node_tag,
                        table_def: self.table_def.clone(),
                    }
                )
            }
            "public.ways" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_way,
                        table_def: self.table_def.clone(),
                    }
                )
            }
            "public.way_nodes" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_way_node,
                        table_def: self.table_def.clone(),
                    }
                )
            }
            "public.way_tags" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_way_tag,
                        table_def: self.table_def.clone(),
                    }
                )
            }
            "public.relations" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_relation,
                        table_def: self.table_def.clone(),
                    }
                )
            }
            "public.relation_members" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_relation_member,
                        table_def: self.table_def.clone(),
                    }
                )
            }
            "public.relation_tags" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_relation_tag,
                        table_def: self.table_def.clone(),
                    }
                )
            }
            "public.changesets" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_changeset,
                        table_def: self.table_def.clone(),
                    }
                )
            }
            "public.users" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_user,
                        table_def: self.table_def.clone(),
                    }
                )
            }
            _ => {
                Err(anyhow!("Unknown record type: {}", self.table_def.name()))
            }
        }
    }

    fn create_node(line: &String, table_def: &TableDef, line_number: usize) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match table_def.fields_ref() {
            TableFields::Nodes { node_id, latitude, longitude, changeset_id, visible, timestamp, tile, version, redaction_id } => {
                assert!(*node_id < columns.len(), "column {} for field (node_id) is missing in {}:{}", *node_id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*latitude < columns.len(), "column {} for field (latitude) is missing in {}:{}", *latitude + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*longitude < columns.len(), "column {} for field (longitude) is missing in {}:{}", *longitude + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*changeset_id < columns.len(), "column {} for field (changeset_id) is missing in {}:{}", *changeset_id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*visible < columns.len(), "column {} for field (visible) is missing in {}:{}", *visible + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*timestamp < columns.len(), "column {} for field (timestamp) is missing in {}:{}", *timestamp + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*tile < columns.len(), "column {} for field (tile) is missing in {}:{}", *tile + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*version < columns.len(), "column {} for field (version) is missing in {}:{}", *version + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*redaction_id < columns.len(), "column {} for field (redaction_id) is missing in {}:{}", *redaction_id + 1, table_def.path().to_string_lossy(), line_number);
                Some(
                    TableRecord::Node {
                        node_record: NodeRecord::new(
                            i64::from_str(columns[*node_id]).unwrap(),
                            i32::from_str(columns[*latitude]).unwrap(),
                            i32::from_str(columns[*longitude]).unwrap(),
                            i64::from_str(columns[*changeset_id]).unwrap(),
                            parse_sql_bool(columns[*visible]).unwrap(),
                            parse_sql_time(columns[*timestamp]).unwrap(),
                            i64::from_str(columns[*tile]).unwrap(),
                            i64::from_str(columns[*version]).unwrap(),
                            i32::from_str(columns[*redaction_id]).ok(),
                        )
                    }
                )
            }
            _ => {
                None
            }
        }
    }

    fn create_node_tag(line: &String, table_def: &TableDef, line_number: usize) -> Option<TableRecord> {
        let columns: Vec<&str> = line.split("\t").collect();
        match table_def.fields_ref() {
            TableFields::NodeTags { node_id, version, k, v } => {
                assert!(*node_id < columns.len(), "column {} for field (node_id) is missing in {}:{}", *node_id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*version < columns.len(), "column {} for field (version) is missing in {}:{}", *version + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*k < columns.len(), "column {} for field (k) is missing in {}:{}", *k + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*v < columns.len(), "column {} for field (v) is missing in {}:{}", *v + 1, table_def.path().to_string_lossy(), line_number);
                Some(
                    TableRecord::NodeTag {
                        node_tag_record: NodeTagRecord::new(
                            i64::from_str(columns[*node_id]).unwrap(),
                            i64::from_str(columns[*version]).unwrap(),
                            columns[*k].to_string(),
                            columns[*v].to_string(),
                        )
                    }
                )
            }
            _ => {
                None
            }
        }
    }

    fn create_way(line: &String, table_def: &TableDef, line_number: usize) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match table_def.fields_ref() {
            TableFields::Ways { way_id, changeset_id, timestamp, version, visible, redaction_id } => {
                assert!(*way_id < columns.len(), "column {} for field (way_id) is missing in {}:{}", *way_id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*changeset_id < columns.len(), "column {} for field (changeset_id) is missing in {}:{}", *changeset_id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*timestamp < columns.len(), "column {} for field (timestamp) is missing in {}:{}", *timestamp + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*version < columns.len(), "column {} for field (version) is missing in {}:{}", *version + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*visible < columns.len(), "column {} for field (visible) is missing in {}:{}", *visible + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*redaction_id < columns.len(), "column {} for field (redaction_id) is missing in {}:{}", *redaction_id + 1, table_def.path().to_string_lossy(), line_number);
                Some(
                    TableRecord::Way {
                        way_record: WayRecord::new(
                            i64::from_str(columns[*way_id]).unwrap(),
                            i64::from_str(columns[*changeset_id]).unwrap(),
                            parse_sql_time(columns[*timestamp]).unwrap(),
                            i64::from_str(columns[*version]).unwrap(),
                            parse_sql_bool(columns[*visible]).unwrap(),
                            i32::from_str(columns[*redaction_id]).ok(),
                        )
                    }
                )
            }
            _ => {
                None
            }
        }
    }

    fn create_way_node(line: &String, table_def: &TableDef, line_number: usize) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match table_def.fields_ref() {
            TableFields::WayNodes { way_id, node_id, version, sequence_id } => {
                assert!(*way_id < columns.len(), "column {} for field (way_id) is missing in {}:{}", *way_id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*node_id < columns.len(), "column {} for field (node_id) is missing in {}:{}", *node_id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*version < columns.len(), "column {} for field (version) is missing in {}:{}", *version + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*sequence_id < columns.len(), "column {} for field (sequence_id) is missing in {}:{}", *sequence_id + 1, table_def.path().to_string_lossy(), line_number);
                Some(
                    TableRecord::WayNode {
                        way_node_record: WayNodeRecord::new(
                            i64::from_str(columns[*way_id]).unwrap(),
                            i64::from_str(columns[*node_id]).unwrap(),
                            i64::from_str(columns[*version]).unwrap(),
                            i64::from_str(columns[*sequence_id]).unwrap(),
                        )
                    }
                )
            }
            _ => {
                None
            }
        }
    }

    fn create_way_tag(line: &String, table_def: &TableDef, line_number: usize) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match table_def.fields_ref() {
            TableFields::WayTags { way_id, k, v, version } => {
                assert!(*way_id < columns.len(), "column {} for field (way_id) is missing in {}:{}", *way_id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*version < columns.len(), "column {} for field (version) is missing in {}:{}", *version + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*k < columns.len(), "column {} for field (k) is missing in {}:{}", *k + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*v < columns.len(), "column {} for field (v) is missing in {}:{}", *v + 1, table_def.path().to_string_lossy(), line_number);
                Some(
                    TableRecord::WayTag {
                        way_tag_record: WayTagRecord::new(
                            i64::from_str(columns[*way_id]).unwrap(),
                            i64::from_str(columns[*version]).unwrap(),
                            columns[*k].to_string(),
                            columns[*v].to_string(),
                        )
                    }
                )
            }
            _ => {
                None
            }
        }
    }

    fn create_relation(line: &String, table_def: &TableDef, line_number: usize) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match table_def.fields_ref() {
            TableFields::Relations { relation_id, changeset_id, timestamp, version, visible, redaction_id } => {
                assert!(*relation_id < columns.len(), "column {} for field (relation_id) is missing in {}:{}", *relation_id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*changeset_id < columns.len(), "column {} for field (changeset_id) is missing in {}:{}", *changeset_id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*timestamp < columns.len(), "column {} for field (timestamp) is missing in {}:{}", *timestamp + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*version < columns.len(), "column {} for field (version) is missing in {}:{}", *version + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*visible < columns.len(), "column {} for field (visible) is missing in {}:{}", *visible + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*redaction_id < columns.len(), "column {} for field (redaction_id) is missing in {}:{}", *redaction_id + 1, table_def.path().to_string_lossy(), line_number);
                Some(
                    TableRecord::Relation {
                        relation_record: RelationRecord::new(
                            i64::from_str(columns[*relation_id]).unwrap(),
                            i64::from_str(columns[*changeset_id]).unwrap(),
                            parse_sql_time(columns[*timestamp]).unwrap(),
                            i64::from_str(columns[*version]).unwrap(),
                            parse_sql_bool(columns[*visible]).unwrap(),
                            i32::from_str(columns[*redaction_id]).ok(),
                        )
                    }
                )
            }
            _ => {
                None
            }
        }
    }

    fn create_relation_member(line: &String, table_def: &TableDef, line_number: usize) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match table_def.fields_ref() {
            TableFields::RelationMembers { relation_id, member_type, member_id, member_role, version, sequence_id } => {
                assert!(*relation_id < columns.len(), "column {} for field (relation_id) is missing in {}:{}", *relation_id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*member_type < columns.len(), "column {} for field (member_type) is missing in {}:{}", *member_type + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*member_id < columns.len(), "column {} for field (member_id) is missing in {}:{}", *member_id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*member_role < columns.len(), "column {} for field (member_role) is missing in {}:{}", *member_role + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*version < columns.len(), "column {} for field (version) is missing in {}:{}", *version + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*sequence_id < columns.len(), "column {} for field (sequence_id) is missing in {}:{}", *sequence_id + 1, table_def.path().to_string_lossy(), line_number);
                Some(
                    TableRecord::RelationMember {
                        relation_member_record: RelationMemberRecord::new(
                            i64::from_str(columns[*relation_id]).unwrap(),
                            RelationMemberType::try_from(columns[*member_type]).unwrap(),
                            i64::from_str(columns[*member_id]).unwrap(),
                            columns[*version].to_string(),
                            i64::from_str(columns[*version]).unwrap(),
                            i64::from_str(columns[*sequence_id]).unwrap(),
                        )
                    }
                )
            }
            _ => {
                None
            }
        }
    }

    fn create_relation_tag(line: &String, table_def: &TableDef, line_number: usize) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match table_def.fields_ref() {
            TableFields::RelationTags { relation_id, k, v, version } => {
                assert!(*relation_id < columns.len(), "column {} for field (relation_id) is missing in {}:{}", *relation_id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*version < columns.len(), "column {} for field (version) is missing in {}:{}", *version + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*k < columns.len(), "column {} for field (k) is missing in {}:{}", *k + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*v < columns.len(), "column {} for field (v) is missing in {}:{}", *v + 1, table_def.path().to_string_lossy(), line_number);
                Some(
                    TableRecord::RelationTag {
                        relation_tag_record: RelationTagRecord::new(
                            i64::from_str(columns[*relation_id]).unwrap(),
                            i64::from_str(columns[*version]).unwrap(),
                            columns[*k].to_string(),
                            columns[*v].to_string(),
                        )
                    }
                )
            }
            _ => {
                None
            }
        }
    }

    fn create_changeset(line: &String, table_def: &TableDef, line_number: usize) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match table_def.fields_ref() {
            TableFields::Changesets { id, user_id, created_at, min_lat, max_lat, min_lon, max_lon, closed_at, num_changes } => {
                assert!(*id < columns.len(), "column {} for field (id) is missing in {}:{}", *id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*user_id < columns.len(), "column {} for field (user_id) is missing in {}:{}", *user_id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*created_at < columns.len(), "column {} for field (created_at) is missing in {}:{}", *created_at + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*min_lat < columns.len(), "column {} for field (min_lat) is missing in {}:{}", *min_lat + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*max_lat < columns.len(), "column {} for field (max_lat) is missing in {}:{}", *max_lat + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*min_lon < columns.len(), "column {} for field (min_lon) is missing in {}:{}", *min_lon + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*max_lon < columns.len(), "column {} for field (max_lon) is missing in {}:{}", *max_lon + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*closed_at < columns.len(), "column {} for field (closed_at) is missing in {}:{}", *closed_at + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*num_changes < columns.len(), "column {} for field (num_changes) is missing in {}:{}", *num_changes + 1, table_def.path().to_string_lossy(), line_number);
                Some(
                    TableRecord::Changeset {
                        changeset_record: ChangesetRecord::new(
                            i64::from_str(columns[*id]).unwrap(),
                            i64::from_str(columns[*user_id]).unwrap(),
                            parse_sql_time(columns[*created_at]).unwrap(),
                            i32::from_str(columns[*min_lat]).unwrap(),
                            i32::from_str(columns[*max_lat]).unwrap(),
                            i32::from_str(columns[*min_lon]).unwrap(),
                            i32::from_str(columns[*max_lon]).unwrap(),
                            parse_sql_time(columns[*closed_at]).unwrap(),
                            i32::from_str(columns[*num_changes]).unwrap(),
                        )
                    }
                )
            }
            _ => {
                None
            }
        }
    }

    fn create_user(line: &String, table_def: &TableDef, line_number: usize) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match table_def.fields_ref() {
            TableFields::Users { email, id, pass_crypt, creation_time, display_name, data_public, description, home_lat, home_lon, home_zoom, pass_salt, email_valid, new_email, creation_ip, languages, status, terms_agreed, consider_pd, auth_uid, preferred_editor, terms_seen, description_format, changesets_count, traces_count, diary_entries_count, image_use_gravatar, auth_provider, home_tile, tou_agreed, } => {
                assert!(*email < columns.len(), "column {} for field (email) is missing in {}:{}", *email + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*id < columns.len(), "column {} for field (id) is missing in {}:{}", *id + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*pass_crypt < columns.len(), "column {} for field (pass_crypt) is missing in {}:{}", *pass_crypt + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*creation_time < columns.len(), "column {} for field (creation_time) is missing in {}:{}", *creation_time + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*display_name < columns.len(), "column {} for field (display_name) is missing in {}:{}", *display_name + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*data_public < columns.len(), "column {} for field (data_public) is missing in {}:{}", *data_public + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*description < columns.len(), "column {} for field (description) is missing in {}:{}", *description + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*home_lat < columns.len(), "column {} for field (home_lat) is missing in {}:{}", *home_lat + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*home_lon < columns.len(), "column {} for field (home_lon) is missing in {}:{}", *home_lon + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*home_zoom < columns.len(), "column {} for field (home_zoom) is missing in {}:{}", *home_zoom + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*pass_salt < columns.len(), "column {} for field (pass_salt) is missing in {}:{}", *pass_salt + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*email_valid < columns.len(), "column {} for field (email_valid) is missing in {}:{}", *email_valid + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*new_email < columns.len(), "column {} for field (new_email) is missing in {}:{}", *new_email + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*creation_ip < columns.len(), "column {} for field (creation_ip) is missing in {}:{}", *creation_ip + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*languages < columns.len(), "column {} for field (languages) is missing in {}:{}", *languages + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*status < columns.len(), "column {} for field (status) is missing in {}:{}", *status + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*terms_agreed < columns.len(), "column {} for field (terms_agreed) is missing in {}:{}", *terms_agreed + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*consider_pd < columns.len(), "column {} for field (consider_pd) is missing in {}:{}", *consider_pd + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*auth_uid < columns.len(), "column {} for field (auth_uid) is missing in {}:{}", *auth_uid + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*preferred_editor < columns.len(), "column {} for field (preferred_editor) is missing in {}:{}", *preferred_editor + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*terms_seen < columns.len(), "column {} for field (terms_seen) is missing in {}:{}", *terms_seen + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*description_format < columns.len(), "column {} for field (description_format) is missing in {}:{}", *description_format + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*changesets_count < columns.len(), "column {} for field (changesets_count) is missing in {}:{}", *changesets_count + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*traces_count < columns.len(), "column {} for field (traces_count) is missing in {}:{}", *traces_count + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*diary_entries_count < columns.len(), "column {} for field (diary_entries_count) is missing in {}:{}", *diary_entries_count + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*image_use_gravatar < columns.len(), "column {} for field (image_use_gravatar) is missing in {}:{}", *image_use_gravatar + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*auth_provider < columns.len(), "column {} for field (auth_provider) is missing in {}:{}", *auth_provider + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*home_tile < columns.len(), "column {} for field (home_tile) is missing in {}:{}", *home_tile + 1, table_def.path().to_string_lossy(), line_number);
                assert!(*tou_agreed < columns.len(), "column {} for field (tou_agreed) is missing in {}:{}", *tou_agreed + 1, table_def.path().to_string_lossy(), line_number);
                Some(
                    TableRecord::User {
                        user_record: UserRecord::new(
                            columns[*email].to_string(),
                            i64::from_str(columns[*id]).unwrap(),
                            columns[*pass_crypt].to_string(),
                            parse_sql_time(columns[*creation_time]).unwrap(),
                            columns[*display_name].to_string(),
                            parse_sql_bool(columns[*data_public]).unwrap(),
                            columns[*description].to_string(),
                            f64::from_str(columns[*home_lat]).ok(),
                            f64::from_str(columns[*home_lon]).ok(),
                            i16::from_str(columns[*home_zoom]).unwrap(),
                            parse_sql_null_string(columns[*pass_salt]),
                            parse_sql_bool(columns[*email_valid]).unwrap(),
                            parse_sql_null_string(columns[*new_email]),
                            parse_sql_null_string(columns[*creation_ip]),
                            parse_sql_null_string(columns[*languages]),
                            UserStatus::try_from(columns[*status]).unwrap(),
                            parse_sql_time(columns[*terms_agreed]).ok(),
                            parse_sql_bool(columns[*consider_pd]).unwrap(),
                            parse_sql_null_string(columns[*auth_uid]),
                            parse_sql_null_string(columns[*preferred_editor]),
                            parse_sql_bool(columns[*terms_seen]).unwrap(),
                            FormatEnum::try_from(columns[*description_format]).unwrap(),
                            i32::from_str(columns[*changesets_count]).unwrap(),
                            i32::from_str(columns[*traces_count]).unwrap(),
                            i32::from_str(columns[*diary_entries_count]).unwrap(),
                            parse_sql_bool(columns[*image_use_gravatar]).unwrap(),
                            parse_sql_null_string(columns[*auth_provider]),
                            i64::from_str(columns[*home_tile]).ok(),
                            parse_sql_time(columns[*tou_agreed]).ok(),
                        )
                    }
                )
            }
            _ => {
                None
            }
        }
    }
}

impl IntoIterator for TableReader {
    type Item = TableRecord;
    type IntoIter = TableIterator;

    fn into_iter(self) -> Self::IntoIter {
        TableIterator::new(&self).unwrap()
    }
}

pub(crate) struct TableIterator {
    reader: BufReader<File>,
    record_builder: RecordBuilder,
    line_number: usize,
}

impl TableIterator {
    pub(crate) fn new(table_reader: &TableReader) -> Result<TableIterator, anyhow::Error> {
        log::info!("Create iterator for {} from {:?}", table_reader.table_def.name(), table_reader.table_def.path());
        let f = File::open(&table_reader.table_def.sorted_path())?;
        let reader = BufReader::new(f);
        let record_builder = table_reader.create_record_builder()?;
        Ok(
            TableIterator {
                reader,
                record_builder,
                line_number: 0,
            }
        )
    }
}

impl Iterator for TableIterator {
    type Item = TableRecord;

    fn next(&mut self) -> Option<Self::Item> {
        self.line_number += 1;
        let mut line = String::with_capacity(2048);
        match self.reader.read_line(&mut line) {
            Ok(0) => {
                None
            }
            Ok(_l) => {
                match line.starts_with("\\.") || line.is_empty() || line.starts_with("\n") {
                    false => {
                        self.record_builder.build(&line, self.line_number)
                    }
                    true => {
                        None
                    }
                }
            }
            Err(_) => {
                None
            }
        }
    }
}