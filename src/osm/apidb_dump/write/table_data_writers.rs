use std::collections::HashMap;
use std::path::PathBuf;
use json::JsonValue;
use transient_btree_index::{BtreeConfig, BtreeIndex};
use crate::osm::apidb_dump::write::table_data_writer::TableDataWriter;

pub(crate) struct TableDataWriters {
    pub(crate) acls: TableDataWriter,
    pub(crate) active_storage_attachments: TableDataWriter,
    pub(crate) active_storage_blobs: TableDataWriter,
    pub(crate) active_storage_variant_records: TableDataWriter,
    pub(crate) ar_internal_metadata: TableDataWriter,
    pub(crate) changeset_comments: TableDataWriter,
    pub(crate) changeset_tags: TableDataWriter,
    pub(crate) changesets: TableDataWriter,
    pub(crate) changesets_subscribers: TableDataWriter,
    pub(crate) client_applications: TableDataWriter,
    pub(crate) current_node_tags: TableDataWriter,
    pub(crate) current_nodes: TableDataWriter,
    pub(crate) current_relation_members: TableDataWriter,
    pub(crate) current_relation_tags: TableDataWriter,
    pub(crate) current_relations: TableDataWriter,
    pub(crate) current_way_nodes: TableDataWriter,
    pub(crate) current_way_tags: TableDataWriter,
    pub(crate) current_ways: TableDataWriter,
    pub(crate) delayed_jobs: TableDataWriter,
    pub(crate) diary_comments: TableDataWriter,
    pub(crate) diary_entries: TableDataWriter,
    pub(crate) diary_entry_subscriptions: TableDataWriter,
    pub(crate) friends: TableDataWriter,
    pub(crate) gps_points: TableDataWriter,
    pub(crate) gpx_file_tags: TableDataWriter,
    pub(crate) gpx_files: TableDataWriter,
    pub(crate) issue_comments: TableDataWriter,
    pub(crate) issues: TableDataWriter,
    pub(crate) languages: TableDataWriter,
    pub(crate) messages: TableDataWriter,
    pub(crate) node_tags: TableDataWriter,
    pub(crate) nodes: TableDataWriter,
    pub(crate) note_comments: TableDataWriter,
    pub(crate) notes: TableDataWriter,
    pub(crate) oauth_access_grants: TableDataWriter,
    pub(crate) oauth_access_tokens: TableDataWriter,
    pub(crate) oauth_applications: TableDataWriter,
    pub(crate) oauth_nonces: TableDataWriter,
    pub(crate) oauth_tokens: TableDataWriter,
    pub(crate) redactions: TableDataWriter,
    pub(crate) relation_members: TableDataWriter,
    pub(crate) relation_tags: TableDataWriter,
    pub(crate) relations: TableDataWriter,
    pub(crate) reports: TableDataWriter,
    pub(crate) schema_migrations: TableDataWriter,
    pub(crate) user_blocks: TableDataWriter,
    pub(crate) user_preferences: TableDataWriter,
    pub(crate) user_roles: TableDataWriter,
    pub(crate) user_tokens: TableDataWriter,
    pub(crate) users: TableDataWriter,
    pub(crate) way_nodes: TableDataWriter,
    pub(crate) way_tags: TableDataWriter,
    pub(crate) ways: TableDataWriter,

    pub(crate) user_index: BtreeIndex<i64, String>,
    pub(crate) changeset_user_index: BtreeIndex<i64, i64>,

    pub(crate) user_index_buffer: HashMap<i64, String>,
    pub(crate) changeset_user_index_buffer: HashMap<i64, i64>,
}

impl TableDataWriters {
    pub(crate) fn new(template_mapping: JsonValue, output_path: &PathBuf) -> Result<Self, anyhow::Error> {
        let user_index = BtreeIndex::<i64, String>::with_capacity(BtreeConfig::default(), 0)?;
        let changeset_user_index = BtreeIndex::<i64, i64>::with_capacity(BtreeConfig::default(), 0)?;
        let user_index_buffer = HashMap::<i64, String>::new();
        let changeset_user_index_buffer = HashMap::<i64, i64>::new();

        Ok(TableDataWriters {
            acls: TableDataWriter::new("public.acls".to_string(), template_mapping["public.acls"].to_string(), output_path).unwrap(),
            active_storage_attachments: TableDataWriter::new("public.active_storage_attachments".to_string(), template_mapping["public.active_storage_attachments"].to_string(), output_path).unwrap(),
            active_storage_blobs: TableDataWriter::new("public.active_storage_blobs".to_string(), template_mapping["public.active_storage_blobs"].to_string(), output_path).unwrap(),
            active_storage_variant_records: TableDataWriter::new("public.active_storage_variant_records".to_string(), template_mapping["public.active_storage_variant_records"].to_string(), output_path).unwrap(),
            ar_internal_metadata: TableDataWriter::new("public.ar_internal_metadata".to_string(), template_mapping["public.ar_internal_metadata"].to_string(), output_path).unwrap(),
            changeset_comments: TableDataWriter::new("public.changeset_comments".to_string(), template_mapping["public.changeset_comments"].to_string(), output_path).unwrap(),
            changeset_tags: TableDataWriter::new("public.changeset_tags".to_string(), template_mapping["public.changeset_tags"].to_string(), output_path).unwrap(),
            changesets: TableDataWriter::new("public.changesets".to_string(), template_mapping["public.changesets"].to_string(), output_path).unwrap(),
            changesets_subscribers: TableDataWriter::new("public.changesets_subscribers".to_string(), template_mapping["public.changesets_subscribers"].to_string(), output_path).unwrap(),
            client_applications: TableDataWriter::new("public.client_applications".to_string(), template_mapping["public.client_applications"].to_string(), output_path).unwrap(),
            current_node_tags: TableDataWriter::new("public.current_node_tags".to_string(), template_mapping["public.current_node_tags"].to_string(), output_path).unwrap(),
            current_nodes: TableDataWriter::new("public.current_nodes".to_string(), template_mapping["public.current_nodes"].to_string(), output_path).unwrap(),
            current_relation_members: TableDataWriter::new("public.current_relation_members".to_string(), template_mapping["public.current_relation_members"].to_string(), output_path).unwrap(),
            current_relation_tags: TableDataWriter::new("public.current_relation_tags".to_string(), template_mapping["public.current_relation_tags"].to_string(), output_path).unwrap(),
            current_relations: TableDataWriter::new("public.current_relations".to_string(), template_mapping["public.current_relations"].to_string(), output_path).unwrap(),
            current_way_nodes: TableDataWriter::new("public.current_way_nodes".to_string(), template_mapping["public.current_way_nodes"].to_string(), output_path).unwrap(),
            current_way_tags: TableDataWriter::new("public.current_way_tags".to_string(), template_mapping["public.current_way_tags"].to_string(), output_path).unwrap(),
            current_ways: TableDataWriter::new("public.current_ways".to_string(), template_mapping["public.current_ways"].to_string(), output_path).unwrap(),
            delayed_jobs: TableDataWriter::new("public.delayed_jobs".to_string(), template_mapping["public.delayed_jobs"].to_string(), output_path).unwrap(),
            diary_comments: TableDataWriter::new("public.diary_comments".to_string(), template_mapping["public.diary_comments"].to_string(), output_path).unwrap(),
            diary_entries: TableDataWriter::new("public.diary_entries".to_string(), template_mapping["public.diary_entries"].to_string(), output_path).unwrap(),
            diary_entry_subscriptions: TableDataWriter::new("public.diary_entry_subscriptions".to_string(), template_mapping["public.diary_entry_subscriptions"].to_string(), output_path).unwrap(),
            friends: TableDataWriter::new("public.friends".to_string(), template_mapping["public.friends"].to_string(), output_path).unwrap(),
            gps_points: TableDataWriter::new("public.gps_points".to_string(), template_mapping["public.gps_points"].to_string(), output_path).unwrap(),
            gpx_file_tags: TableDataWriter::new("public.gpx_file_tags".to_string(), template_mapping["public.gpx_file_tags"].to_string(), output_path).unwrap(),
            gpx_files: TableDataWriter::new("public.gpx_files".to_string(), template_mapping["public.gpx_files"].to_string(), output_path).unwrap(),
            issue_comments: TableDataWriter::new("public.issue_comments".to_string(), template_mapping["public.issue_comments"].to_string(), output_path).unwrap(),
            issues: TableDataWriter::new("public.issues".to_string(), template_mapping["public.issues"].to_string(), output_path).unwrap(),
            languages: TableDataWriter::new("public.languages".to_string(), template_mapping["public.languages"].to_string(), output_path).unwrap(),
            messages: TableDataWriter::new("public.messages".to_string(), template_mapping["public.messages"].to_string(), output_path).unwrap(),
            node_tags: TableDataWriter::new("public.node_tags".to_string(), template_mapping["public.node_tags"].to_string(), output_path).unwrap(),
            nodes: TableDataWriter::new("public.nodes".to_string(), template_mapping["public.nodes"].to_string(), output_path).unwrap(),
            note_comments: TableDataWriter::new("public.note_comments".to_string(), template_mapping["public.note_comments"].to_string(), output_path).unwrap(),
            notes: TableDataWriter::new("public.notes".to_string(), template_mapping["public.notes"].to_string(), output_path).unwrap(),
            oauth_access_grants: TableDataWriter::new("public.oauth_access_grants".to_string(), template_mapping["public.oauth_access_grants"].to_string(), output_path).unwrap(),
            oauth_access_tokens: TableDataWriter::new("public.oauth_access_tokens".to_string(), template_mapping["public.oauth_access_tokens"].to_string(), output_path).unwrap(),
            oauth_applications: TableDataWriter::new("public.oauth_applications".to_string(), template_mapping["public.oauth_applications"].to_string(), output_path).unwrap(),
            oauth_nonces: TableDataWriter::new("public.oauth_nonces".to_string(), template_mapping["public.oauth_nonces"].to_string(), output_path).unwrap(),
            oauth_tokens: TableDataWriter::new("public.oauth_tokens".to_string(), template_mapping["public.oauth_tokens"].to_string(), output_path).unwrap(),
            redactions: TableDataWriter::new("public.redactions".to_string(), template_mapping["public.redactions"].to_string(), output_path).unwrap(),
            relation_members: TableDataWriter::new("public.relation_members".to_string(), template_mapping["public.relation_members"].to_string(), output_path).unwrap(),
            relation_tags: TableDataWriter::new("public.relation_tags".to_string(), template_mapping["public.relation_tags"].to_string(), output_path).unwrap(),
            relations: TableDataWriter::new("public.relations".to_string(), template_mapping["public.relations"].to_string(), output_path).unwrap(),
            reports: TableDataWriter::new("public.reports".to_string(), template_mapping["public.reports"].to_string(), output_path).unwrap(),
            schema_migrations: TableDataWriter::new("public.schema_migrations".to_string(), template_mapping["public.schema_migrations"].to_string(), output_path).unwrap(),
            user_blocks: TableDataWriter::new("public.user_blocks".to_string(), template_mapping["public.user_blocks"].to_string(), output_path).unwrap(),
            user_preferences: TableDataWriter::new("public.user_preferences".to_string(), template_mapping["public.user_preferences"].to_string(), output_path).unwrap(),
            user_roles: TableDataWriter::new("public.user_roles".to_string(), template_mapping["public.user_roles"].to_string(), output_path).unwrap(),
            user_tokens: TableDataWriter::new("public.user_tokens".to_string(), template_mapping["public.user_tokens"].to_string(), output_path).unwrap(),
            users: TableDataWriter::new("public.users".to_string(), template_mapping["public.users"].to_string(), output_path).unwrap(),
            way_nodes: TableDataWriter::new("public.way_nodes".to_string(), template_mapping["public.way_nodes"].to_string(), output_path).unwrap(),
            way_tags: TableDataWriter::new("public.way_tags".to_string(), template_mapping["public.way_tags"].to_string(), output_path).unwrap(),
            ways: TableDataWriter::new("public.ways".to_string(), template_mapping["public.ways"].to_string(), output_path).unwrap(),

            user_index,
            changeset_user_index,
            user_index_buffer,
            changeset_user_index_buffer,
        })
    }

    pub(crate) fn flush_buffers(&mut self) -> Result<(), anyhow::Error> {
        self.user_index_buffer.iter().for_each(|(user_id, user)| {
            self.user_index.insert(*user_id, user.to_string()).unwrap();
        });
        self.user_index_buffer.clear();

        self.changeset_user_index_buffer.iter().for_each(|(changeset_id, user_id)| {
            self.changeset_user_index.insert(*changeset_id, *user_id).unwrap();
        });
        self.changeset_user_index_buffer.clear();
        Ok(())
    }

    pub(crate) fn close(&mut self) -> Result<(), anyhow::Error>{
        self.acls.close()?;
        self.active_storage_attachments.close()?;
        self.active_storage_blobs.close()?;
        self.active_storage_variant_records.close()?;
        self.ar_internal_metadata.close()?;
        self.changeset_comments.close()?;
        self.changeset_tags.close()?;
        self.changesets.close()?;
        self.changesets_subscribers.close()?;
        self.client_applications.close()?;
        self.current_node_tags.close()?;
        self.current_nodes.close()?;
        self.current_relation_members.close()?;
        self.current_relation_tags.close()?;
        self.current_relations.close()?;
        self.current_way_nodes.close()?;
        self.current_way_tags.close()?;
        self.current_ways.close()?;
        self.delayed_jobs.close()?;
        self.diary_comments.close()?;
        self.diary_entries.close()?;
        self.diary_entry_subscriptions.close()?;
        self.friends.close()?;
        self.gps_points.close()?;
        self.gpx_file_tags.close()?;
        self.gpx_files.close()?;
        self.issue_comments.close()?;
        self.issues.close()?;
        self.languages.close()?;
        self.messages.close()?;
        self.node_tags.close()?;
        self.nodes.close()?;
        self.note_comments.close()?;
        self.notes.close()?;
        self.oauth_access_grants.close()?;
        self.oauth_access_tokens.close()?;
        self.oauth_applications.close()?;
        self.oauth_nonces.close()?;
        self.oauth_tokens.close()?;
        self.redactions.close()?;
        self.relation_members.close()?;
        self.relation_tags.close()?;
        self.relations.close()?;
        self.reports.close()?;
        self.schema_migrations.close()?;
        self.user_blocks.close()?;
        self.user_preferences.close()?;
        self.user_roles.close()?;
        self.user_tokens.close()?;
        self.users.close()?;
        self.way_nodes.close()?;
        self.way_tags.close()?;
        self.ways.close()?;
        Ok(())
    }
}
