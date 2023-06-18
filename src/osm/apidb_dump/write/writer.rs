use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Error};
use escape_string::escape;

use crate::osm::apidb_dump::sql::{calculate_tile, to_sql_bool, to_sql_time, to_sql_time_micro};
use crate::osm::apidb_dump::write::current_object::{CurrentObjectLine, CurrentObjectLines};
use crate::osm::apidb_dump::write::table_data_writers::TableDataWriters;
use crate::osm::apidb_dump::write::toc::{load_template_mapping, write_toc};
use crate::osm::model::element::Element;
use crate::osm::model::node::Node;
use crate::osm::model::relation::{Member, Relation};
use crate::osm::model::way::Way;

/// Writer of apidb schema dump
///
/// Writer of apidb schema dump that can be loaded using pg_restore into a Postgresql database
pub struct Writer {
    #[allow(dead_code)]
    output_path: PathBuf,
    #[allow(dead_code)]
    compression_level: i8,
    writers: TableDataWriters,
    current_node_line: CurrentObjectLine,
    current_node_tag_lines: CurrentObjectLines,
    current_way_line: CurrentObjectLine,
    current_way_node_lines: CurrentObjectLines,
    current_way_tag_lines: CurrentObjectLines,
    current_relation_line: CurrentObjectLine,
    current_relation_member_lines: CurrentObjectLines,
    current_relation_tag_lines: CurrentObjectLines,
}

impl Writer {
    /// Create a new [Writer]
    ///
    /// * output_path - directory to write the output to. Must contain enough space which is very
    /// difficult to calculate because the *.osm.pbf input is so condensed that 1GB of input can
    /// easily transform into 100GB of output.
    pub fn new(output_path: PathBuf, compression_level: i8) -> Result<Writer, anyhow::Error> {
        Self::create_result_dir(&output_path)?;
        let writers = TableDataWriters::new(load_template_mapping()?, &output_path)?;
        Ok(
            Writer {
                output_path,
                compression_level,
                writers,
                current_node_line: CurrentObjectLine::new(),
                current_node_tag_lines: CurrentObjectLines::new(),
                current_way_line: CurrentObjectLine::new(),
                current_way_node_lines: CurrentObjectLines::new(),
                current_way_tag_lines: CurrentObjectLines::new(),
                current_relation_line: CurrentObjectLine::new(),
                current_relation_member_lines: CurrentObjectLines::new(),
                current_relation_tag_lines: CurrentObjectLines::new(),
            }
        )
    }

    /// Write an element
    pub fn write_element(&mut self, element: Element) -> Result<(), anyhow::Error> {
        match element {
            Element::Node { node } => {
                self.write_node(node)?;
            }
            Element::Way { way } => {
                self.write_way(way)?;
            }
            Element::Relation { relation } => {
                self.write_relation(relation)?;
            }
            Element::Sentinel => {}
        }
        Ok(())
    }

    fn write_node(&mut self, mut node: Node) -> Result<(), Error> {
        self.writers.user_index_buffer.insert(node.uid() as i64, node.take_user());
        self.writers.changeset_user_index_buffer.insert(node.changeset(), node.uid() as i64);

        // public.current_nodes (id, latitude, longitude, changeset_id, visible, "timestamp", tile, version)
        // template context: 4228.dat
        let current_node_line = format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                                        node.id(),
                                        node.coordinate().lat7(),
                                        node.coordinate().lon7(),
                                        node.changeset(),
                                        to_sql_bool(node.visible()),
                                        to_sql_time(node.timestamp()),
                                        calculate_tile(node.coordinate().lat(), node.coordinate().lon()),
                                        node.version()
        );

        match self.current_node_line.set_last_line(current_node_line, node.id(), node.visible()) {
            None => {}
            Some(current_node_line) => {
                self.writers.current_nodes.writer().write(current_node_line.as_bytes())?;
            }
        }
        self.current_node_line.set_last_id(node.id());

        // public.nodes (node_id, latitude, longitude, changeset_id, visible, "timestamp", tile, version, redaction_id)
        // template context: 4260.dat
        let node_line = format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t\\N\n",
                                node.id(),
                                node.coordinate().lat7(),
                                node.coordinate().lon7(),
                                node.changeset(),
                                to_sql_bool(node.visible()),
                                to_sql_time(node.timestamp()),
                                calculate_tile(node.coordinate().lat(), node.coordinate().lon()),
                                node.version()
        );

        self.writers.nodes.writer().write(node_line.as_bytes())?;

        let mut current_node_tag_lines = Vec::new();
        let tags = node.take_tags();
        for tag in tags {
            // public.node_tags (node_id, version, k, v)
            // template context: 4259.dat
            let escaped_tag = escape(&tag.v());
            let node_tag_line = format!("{}\t{}\t{}\t{}\n",
                                        node.id(),
                                        node.version(),
                                        tag.k(),
                                        escaped_tag,
            );
            self.writers.node_tags.writer().write(node_tag_line.as_bytes())?;

            // public.current_node_tags (node_id, k, v)
            // template context: 4227.dat
            let current_node_tag_line = format!("{}\t{}\t{}\n",
                                                node.id(),
                                                tag.k(),
                                                escaped_tag,
            );
            // self.writers.current_node_tags.writer().write(current_node_tag_line.as_bytes())?;
            current_node_tag_lines.push(current_node_tag_line);
        }

        match self.current_node_tag_lines.set_last_lines(current_node_tag_lines, node.id(), node.visible()) {
            None => {}
            Some(current_node_tag_lines) => {
                for current_node_tag_line in current_node_tag_lines {
                    self.writers.current_node_tags.writer().write(current_node_tag_line.as_bytes())?;
                }
            }
        }
        self.current_node_tag_lines.set_last_id(node.id());

        Ok(())
    }

    fn write_way(&mut self, mut way: Way) -> Result<(), Error> {
        self.writers.user_index_buffer.insert(way.uid() as i64, way.take_user());
        self.writers.changeset_user_index_buffer.insert(way.changeset(), way.uid() as i64);


        let mut current_way_node_lines = Vec::new();
        for (sequence_id, node_id) in way.refs().iter().enumerate() {
            // public.current_way_nodes (way_id, node_id, sequence_id)
            // template context: 4234.dat
            let current_way_node_line = format!("{}\t{}\t{}\n",
                                                way.id(),
                                                node_id,
                                                sequence_id + 1
            );
            current_way_node_lines.push(current_way_node_line);

            // public.way_nodes (way_id, node_id, version, sequence_id)
            // template context: 4292.dat
            let way_node_line = format!("{}\t{}\t{}\t{}\n",
                                        way.id(),
                                        node_id,
                                        way.version(),
                                        sequence_id + 1
            );
            self.writers.way_nodes.writer().write(way_node_line.as_bytes())?;
        }

        match self.current_way_node_lines.set_last_lines(current_way_node_lines, way.id(), way.visible()) {
            None => {}
            Some(current_way_node_lines) => {
                for current_way_node_line in current_way_node_lines {
                    self.writers.current_way_nodes.writer().write(current_way_node_line.as_bytes())?;
                }
            }
        }
        self.current_way_node_lines.set_last_id(way.id());


        let mut current_way_tag_lines = Vec::new();
        for tag in way.take_tags() {
            // public.current_way_tags (way_id, k, v)
            // template context: 4235.dat
            let escaped_tag = escape_string::escape(tag.v());
            let current_way_tag_line = format!("{}\t{}\t{}\n",
                                               way.id(),
                                               tag.k(),
                                               escaped_tag,
            );
            current_way_tag_lines.push(current_way_tag_line);

            // public.way_tags (way_id, k, v, version)
            // template context: 4293.dat
            let way_tag_line_line = format!("{}\t{}\t{}\t{}\n",
                                            way.id(),
                                            tag.k(),
                                            escaped_tag,
                                            way.version()
            );
            self.writers.way_tags.writer().write(way_tag_line_line.as_bytes())?;
        }

        match self.current_way_tag_lines.set_last_lines(current_way_tag_lines, way.id(), way.visible()) {
            None => {}
            Some(current_way_tag_lines) => {
                for current_way_tag_line in current_way_tag_lines {
                    self.writers.current_way_tags.writer().write(current_way_tag_line.as_bytes())?;
                }
            }
        }
        self.current_way_tag_lines.set_last_id(way.id());

        // public.current_ways (id, changeset_id, "timestamp", visible, version)
        // template context: 4236.dat
        let current_way_line = format!("{}\t{}\t{}\t{}\t{}\n",
                                       way.id(),
                                       way.changeset(),
                                       to_sql_time(way.timestamp()),
                                       to_sql_bool(way.visible()),
                                       way.version(),
        );

        match self.current_way_line.set_last_line(current_way_line, way.id(), way.visible()) {
            None => {}
            Some(current_way_line) => {
                self.writers.current_ways.writer().write(current_way_line.as_bytes())?;
            }
        }
        self.current_way_line.set_last_id(way.id());

        // public.ways (way_id, changeset_id, "timestamp", version, visible, redaction_id)
        // template context: 4294.dat"
        let way_line = format!("{}\t{}\t{}\t{}\t{}\t\\N\n",
                               way.id(),
                               way.changeset(),
                               to_sql_time(way.timestamp()),
                               way.version(),
                               to_sql_bool(way.visible()),
        );
        self.writers.ways.writer().write(way_line.as_bytes())?;

        Ok(())
    }

    fn write_relation(&mut self, mut relation: Relation) -> Result<(), Error> {
        self.writers.user_index_buffer.insert(relation.uid() as i64, relation.take_user());
        self.writers.changeset_user_index_buffer.insert(relation.changeset(), relation.uid() as i64);
        let mut current_relation_member_lines = Vec::new();
        for (sequence_id, member) in relation.members().iter().enumerate() {
            let (member_id, member_role, member_type) = match member {
                Member::Node { member } => {
                    (member.id(), member.role(), "Node")
                }
                Member::Way { member } => {
                    (member.id(), member.role(), "Way")
                }
                Member::Relation { member } => {
                    (member.id(), member.role(), "Relation")
                }
            };

            // public.current_relation_members (relation_id, member_type, member_id, member_role, sequence_id)
            // template context: 4230.dat
            let escaped_role = escape_string::escape(member_role);
            let current_relation_member_line = format!("{}\t{}\t{}\t{}\t{}\n",
                                                       relation.id(),
                                                       member_type,
                                                       member_id,
                                                       escaped_role,
                                                       sequence_id + 1,
            );
            current_relation_member_lines.push(current_relation_member_line);

            // public.relation_members (relation_id, member_type, member_id, member_role, version, sequence_id)
            // template context: 4277.dat
            let relation_member_line = format!("{}\t{}\t{}\t{}\t{}\t{}\n",
                                               relation.id(),
                                               member_type,
                                               member_id,
                                               escaped_role,
                                               relation.version(),
                                               sequence_id + 1,
            );
            self.writers.relation_members.writer().write(relation_member_line.as_bytes())?;
        }

        match self.current_relation_member_lines.set_last_lines(current_relation_member_lines, relation.id(), relation.visible()) {
            None => {}
            Some(current_relation_member_lines) => {
                for current_relation_member_line in current_relation_member_lines {
                    self.writers.current_relation_members.writer().write(current_relation_member_line.as_bytes())?;
                }
            }
        }
        self.current_relation_member_lines.set_last_id(relation.id());

        let mut current_relation_tag_lines = Vec::new();
        for tag in relation.take_tags() {
            // public.current_relation_tags (relation_id, k, v)
            // template context: 4231.dat
            let escaped_tag = escape_string::escape(&tag.v());
            let current_relation_tag_line = format!("{}\t{}\t{}\n",
                                                    relation.id(),
                                                    tag.k(),
                                                    escaped_tag,
            );
            current_relation_tag_lines.push(current_relation_tag_line);

            // public.relation_tags (relation_id, k, v, version)
            // template context: 4278.dat
            let relation_tag_line = format!("{}\t{}\t{}\t{}\n",
                                            relation.id(),
                                            tag.k(),
                                            escaped_tag,
                                            relation.version(),
            );
            self.writers.relation_tags.writer().write(relation_tag_line.as_bytes())?;
        }

        match self.current_relation_tag_lines.set_last_lines(current_relation_tag_lines, relation.id(), relation.visible()) {
            None => {}
            Some(current_relation_tag_lines) => {
                for current_relation_tag_line in current_relation_tag_lines {
                    self.writers.current_relation_tags.writer().write(current_relation_tag_line.as_bytes())?;
                }
            }
        }
        self.current_relation_tag_lines.set_last_id(relation.id());


        // public.current_relations (id, changeset_id, "timestamp", visible, version)
        // template context: 4232.dat
        let current_relation_line = format!("{}\t{}\t{}\t{}\t{}\n",
                                            relation.id(),
                                            relation.changeset(),
                                            to_sql_time(relation.timestamp()),
                                            to_sql_bool(relation.visible()),
                                            relation.version(),
        );

        match self.current_relation_line.set_last_line(current_relation_line, relation.id(), relation.visible()) {
            None => {}
            Some(current_relation_line) => {
                self.writers.current_relations.writer().write(current_relation_line.as_bytes())?;
            }
        }
        self.current_relation_line.set_last_id(relation.id());

        // public.relations (relation_id, changeset_id, "timestamp", version, visible, redaction_id)
        // template context: 4279.dat
        let relation_line = format!("{}\t{}\t{}\t{}\t{}\t\\N\n",
                                    relation.id(),
                                    relation.changeset(),
                                    to_sql_time(relation.timestamp()),
                                    relation.version(),
                                    to_sql_bool(relation.visible()),
        );
        self.writers.relations.writer().write(relation_line.as_bytes())?;

        Ok(())
    }

    fn write_changesets(&mut self) -> Result<(), Error> {
        for element in self.writers.changeset_user_index.range(..)? {
            let (changeset_id, user_id) = element?;
            // public.changeset_tags (changeset_id, k, v)
            // template context: 4221.dat
            let line = format!("{}\t{}\t{}\n",
                               changeset_id,
                               "created_by",
                               format!("osm-io {}", option_env!("CARGO_PKG_VERSION").unwrap()),
            );
            self.writers.changeset_tags.writer().write(line.as_bytes())?;

            let line = format!("{}\t{}\t{}\n",
                               changeset_id,
                               "replication",
                               "true"
            );
            self.writers.changeset_tags.writer().write(line.as_bytes())?;

            // public.changesets (id, user_id, created_at, min_lat, max_lat, min_lon, max_lon, closed_at, num_changes)
            // template context: 4222.dat
            let t = chrono::offset::Utc::now();
            let line = format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                               changeset_id,
                               user_id,
                               to_sql_time_micro(t.timestamp_nanos()),
                               -900000000,
                               900000000,
                               -1800000000,
                               1800000000,
                               to_sql_time_micro(t.timestamp_nanos()),
                               0
            );
            self.writers.changesets.writer().write(line.as_bytes())?;
        }

        Ok(())
    }

    fn write_users(&mut self) -> Result<(), Error> {
        // public.users (email, id, pass_crypt, creation_time, display_name, data_public, description, home_lat, home_lon, home_zoom, pass_salt, email_valid, new_email, creation_ip, languages, status, terms_agreed, consider_pd, auth_uid, preferred_editor, terms_seen, description_format, changesets_count, traces_count, diary_entries_count, image_use_gravatar, auth_provider, home_tile, tou_agreed)
        // template context: 4290.dat
        for element in self.writers.user_index.range(..)? {
            let (user_id, user_name) = element?;

            let t = chrono::offset::Utc::now();
            let line = format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                               format!("osm-admin-user-{}@example.com", user_id),
                               user_id,
                               "00000000000000000000000000000000",
                               to_sql_time_micro(t.timestamp_nanos()),
                               user_name,
                               to_sql_bool(true),
                               user_name,
                               0,
                               0,
                               3,
                               "00000000",
                               to_sql_bool(false),
                               "\\N",
                               "\\N",
                               "\\N",
                               "pending",
                               "\\N",
                               to_sql_bool(false),
                               "\\N",
                               "\\N",
                               to_sql_bool(false),
                               "markdown",
                               0,
                               0,
                               0,
                               to_sql_bool(false),
                               "\\N",
                               "\\N",
                               "\\N",
            );
            self.writers.users.writer().write(line.as_bytes())?;
        }

        Ok(())
    }

    fn flush_current_object_lines(&mut self) -> Result<(), Error> {
        match self.current_node_line.take() {
            None => {}
            Some(current_node_line) => {
                self.writers.current_nodes.writer().write(current_node_line.as_bytes())?;
            }
        }

        match self.current_node_tag_lines.take() {
            None => {}
            Some(current_node_tag_lines) => {
                for current_node_tag_line in current_node_tag_lines {
                    self.writers.current_node_tags.writer().write(current_node_tag_line.as_bytes())?;
                }
            }
        }

        match self.current_way_line.take() {
            None => {}
            Some(current_way_line) => {
                self.writers.current_ways.writer().write(current_way_line.as_bytes())?;
            }
        }

        match self.current_way_tag_lines.take() {
            None => {}
            Some(current_way_tag_lines) => {
                for current_way_tag_line in current_way_tag_lines {
                    self.writers.current_way_tags.writer().write(current_way_tag_line.as_bytes())?;
                }
            }
        }

        match self.current_way_node_lines.take() {
            None => {}
            Some(current_way_node_lines) => {
                for current_way_node_line in current_way_node_lines {
                    self.writers.current_way_nodes.writer().write(current_way_node_line.as_bytes())?;
                }
            }
        }

        match self.current_relation_line.take() {
            None => {}
            Some(current_relation_line) => {
                self.writers.current_relations.writer().write(current_relation_line.as_bytes())?;
            }
        }

        match self.current_relation_tag_lines.take() {
            None => {}
            Some(current_relation_tag_lines) => {
                for current_relation_tag_line in current_relation_tag_lines {
                    self.writers.current_relation_tags.writer().write(current_relation_tag_line.as_bytes())?;
                }
            }
        }

        match self.current_relation_member_lines.take() {
            None => {}
            Some(current_relation_member_lines) => {
                for current_relation_member_line in current_relation_member_lines {
                    self.writers.current_relation_members.writer().write(current_relation_member_line.as_bytes())?;
                }
            }
        }

        Ok(())
    }

    /// Flush internal buffers and add file terminators
    pub fn close(&mut self) -> Result<(), Error> {
        self.flush_current_object_lines()?;
        self.writers.flush_buffers()?;
        self.write_users()?;
        self.write_changesets()?;
        self.writers.close()?;
        Ok(())
    }

    /// Return table to file mapping for diagnostics
    pub fn table_mapping(&self) -> Vec<String> {
        Vec::new()
    }

    fn create_result_dir(output_path: &PathBuf) -> Result<(), Error> {
        fs::create_dir_all(&output_path).with_context(|| format!("Failed to create dir: {:?}", output_path))?;
        write_toc(output_path)?;

        Ok(())
    }
}