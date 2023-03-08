use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use num_format::Locale::se;
use postgres::types::IsNull::No;

use crate::osm::apidb_dump::changeset_record::ChangesetRecord;
use crate::osm::apidb_dump::node_record::NodeRecord;
use crate::osm::apidb_dump::node_tag_record::NodeTagRecord;
use crate::osm::apidb_dump::sql::{parse_sql_bool, parse_sql_null_string, parse_sql_time};
use crate::osm::apidb_dump::table_def::TableDef;
use crate::osm::apidb_dump::table_fields::TableFields;
use crate::osm::apidb_dump::table_record::{TableRecord};
use crate::osm::apidb_dump::user_record::{FormatEnum, UserRecord, UserStatus};
use crate::error::{GenericError, OsmIoError};
use crate::osm::apidb_dump::relation_member_record::{RelationMemberRecord, RelationMemberType};
use crate::osm::apidb_dump::relation_record::RelationRecord;
use crate::osm::apidb_dump::relation_tag_record::RelationTagRecord;
use crate::osm::apidb_dump::way_node_record::WayNodeRecord;
use crate::osm::apidb_dump::way_record::WayRecord;
use crate::osm::apidb_dump::way_tag_record::WayTagRecord;

struct RecordBuilder {
    f: fn(&String, &TableFields) -> Option<TableRecord>,
    fields_variant: TableFields,
}

impl RecordBuilder {
    fn build(&self, line: &String) -> Option<TableRecord> {
        (self.f)(line, &self.fields_variant)
    }
}

#[derive(Clone)]
pub(crate) struct TableReader {
    table_def: TableDef,
}

impl TableReader {
    pub(crate) fn new(table_def: &TableDef) -> Result<TableReader, GenericError> {
        Ok(
            TableReader {
                table_def: table_def.clone(),
            }
        )
    }

    fn create_record_builder(&self) -> Result<RecordBuilder, GenericError> {
        match self.table_def.name().as_str() {
            "public.nodes" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_node,
                        fields_variant: self.table_def.fields(),
                    }
                )
            }
            "public.node_tags" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_node_tag,
                        fields_variant: self.table_def.fields(),
                    }
                )
            }
            "public.ways" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_way,
                        fields_variant: self.table_def.fields(),
                    }
                )
            }
            "public.way_nodes" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_way_node,
                        fields_variant: self.table_def.fields(),
                    }
                )
            }
            "public.way_tags" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_way_tag,
                        fields_variant: self.table_def.fields(),
                    }
                )
            }
            "public.relations" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_relation,
                        fields_variant: self.table_def.fields(),
                    }
                )
            }
            "public.relation_members" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_relation_member,
                        fields_variant: self.table_def.fields(),
                    }
                )
            }
            "public.relation_tags" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_relation_tag,
                        fields_variant: self.table_def.fields(),
                    }
                )
            }
            "public.changesets" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_changeset,
                        fields_variant: self.table_def.fields(),
                    }
                )
            }
            "public.users" => {
                Ok(
                    RecordBuilder {
                        f: Self::create_user,
                        fields_variant: self.table_def.fields(),
                    }
                )
            }
            _ => {
                Err(OsmIoError::as_generic(format!("Unknown record type: {}", self.table_def.name())))
            }
        }
    }

    fn create_node(line: &String, fields: &TableFields) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match fields {
            TableFields::Nodes { node_id, latitude, longitude, changeset_id, visible, timestamp, tile, version, redaction_id } => {
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

    fn create_node_tag(line: &String, fields: &TableFields) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match fields {
            TableFields::NodeTags { node_id, version, k, v } => {
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

    fn create_way(line: &String, fields: &TableFields) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match fields {
            TableFields::Ways { way_id, changeset_id, timestamp, version, visible, redaction_id } => {
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

    fn create_way_node(line: &String, fields: &TableFields) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match fields {
            TableFields::WayNodes { way_id, node_id, version, sequence_id } => {
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

    fn create_way_tag(line: &String, fields: &TableFields) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match fields {
            TableFields::WayTags { way_id, k, v, version } => {
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

    fn create_relation(line: &String, fields: &TableFields) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match fields {
            TableFields::Relations { relation_id, changeset_id, timestamp, version, visible, redaction_id } => {
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

    fn create_relation_member(line: &String, fields: &TableFields) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match fields {
            TableFields::RelationMembers { relation_id, member_type, member_id, member_role, version, sequence_id } => {
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

    fn create_relation_tag(line: &String, fields: &TableFields) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match fields {
            TableFields::RelationTags { relation_id, k, v, version } => {
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

    fn create_changeset(line: &String, fields: &TableFields) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match fields {
            TableFields::Changesets { id, user_id, created_at, min_lat, max_lat, min_lon, max_lon, closed_at, num_changes } => {
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

    fn create_user(line: &String, fields: &TableFields) -> Option<TableRecord> {
        let columns: Vec<&str> = line.trim().split("\t").collect();
        match fields {
            TableFields::Users { email, id, pass_crypt, creation_time, display_name, data_public, description, home_lat, home_lon, home_zoom, pass_salt, email_valid, new_email, creation_ip, languages, status, terms_agreed, consider_pd, auth_uid, preferred_editor, terms_seen, description_format, changesets_count, traces_count, diary_entries_count, image_use_gravatar, auth_provider, home_tile, tou_agreed, } => {
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
}

impl TableIterator {
    pub(crate) fn new(table_reader: &TableReader) -> Result<TableIterator, GenericError> {
        log::info!("Create iterator for {} from {:?}", table_reader.table_def.name(), table_reader.table_def.path());
        let f = File::open(&table_reader.table_def.path())?;
        let mut reader = BufReader::new(f);
        let record_builder = table_reader.create_record_builder()?;
        Ok(
            TableIterator {
                reader,
                record_builder,
            }
        )
    }
}

impl Iterator for TableIterator {
    type Item = TableRecord;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::with_capacity(2048);
        match self.reader.read_line(&mut line) {
            Ok(0) => {
                None
            }
            Ok(l) => {
                match line.starts_with("\\.") || line.is_empty() || line.starts_with("\n") {
                    false => {
                        self.record_builder.build(&line)
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