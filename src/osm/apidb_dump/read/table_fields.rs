use anyhow::anyhow;
use crate::osm::apidb_dump::read::table_fields::TableFields::{Changesets, NodeTags, RelationMembers, Relations, RelationTags, Users, WayNodes, Ways, WayTags};

#[derive(Debug, Copy, Clone)]
pub enum TableFields {
    Nodes {
        node_id: usize,
        latitude: usize,
        longitude: usize,
        changeset_id: usize,
        visible: usize,
        timestamp: usize,
        tile: usize,
        version: usize,
        redaction_id: usize,
    },
    NodeTags {
        node_id: usize,
        version: usize,
        k: usize,
        v: usize,
    },
    Ways {
        way_id: usize,
        changeset_id: usize,
        timestamp: usize,
        version: usize,
        visible: usize,
        redaction_id: usize,
    },
    WayTags {
        way_id: usize,
        k: usize,
        v: usize,
        version: usize,
    },
    WayNodes {
        way_id: usize,
        node_id: usize,
        version: usize,
        sequence_id: usize,
    },
    Relations {
        relation_id: usize,
        changeset_id: usize,
        timestamp: usize,
        version: usize,
        visible: usize,
        redaction_id: usize,
    },
    RelationTags {
        relation_id: usize,
        k: usize,
        v: usize,
        version: usize,
    },
    RelationMembers {
        relation_id: usize,
        member_type: usize,
        member_id: usize,
        member_role: usize,
        version: usize,
        sequence_id: usize,
    },
    Changesets {
        id: usize,
        user_id: usize,
        created_at: usize,
        min_lat: usize,
        max_lat: usize,
        min_lon: usize,
        max_lon: usize,
        closed_at: usize,
        num_changes: usize,
    },
    Users {
        email: usize,
        id: usize,
        pass_crypt: usize,
        creation_time: usize,
        display_name: usize,
        data_public: usize,
        description: usize,
        home_lat: usize,
        home_lon: usize,
        home_zoom: usize,
        pass_salt: usize,
        email_valid: usize,
        new_email: usize,
        creation_ip: usize,
        languages: usize,
        status: usize,
        terms_agreed: usize,
        consider_pd: usize,
        auth_uid: usize,
        preferred_editor: usize,
        terms_seen: usize,
        description_format: usize,
        changesets_count: usize,
        traces_count: usize,
        diary_entries_count: usize,
        image_use_gravatar: usize,
        auth_provider: usize,
        home_tile: usize,
        tou_agreed: usize,
    },
}

impl TableFields {
    fn index(v: &str, fields: &Vec<String>) -> Result<usize, anyhow::Error> {
        match fields.iter().position(|e| { *e == v.to_string() }) {
            None => {
                Err(anyhow!("Field not found: {}", v))
            }
            Some(i) => {
                Ok(i)
            }
        }
    }

    pub fn is_of_interest(name: &str) -> bool {
        let tables_of_interest = vec![
            "public.nodes",
            "public.node_tags",
            "public.ways",
            "public.way_nodes",
            "public.way_tags",
            "public.relations",
            "public.relation_members",
            "public.relation_tags",
            "public.changesets",
            "public.users",
        ];

        tables_of_interest.contains(&name)
    }

    pub fn new(name: String, fields: Vec<String>) -> Result<TableFields, anyhow::Error> {
        match name.as_str() {
            "public.nodes" => {
                Ok(
                    TableFields::Nodes {
                        node_id: Self::index("node_id", &fields)?,
                        latitude: Self::index("latitude", &fields)?,
                        longitude: Self::index("longitude", &fields)?,
                        changeset_id: Self::index("changeset_id", &fields)?,
                        visible: Self::index("visible", &fields)?,
                        timestamp: Self::index("\"timestamp\"", &fields)?,
                        tile: Self::index("tile", &fields)?,
                        version: Self::index("version", &fields)?,
                        redaction_id: Self::index("redaction_id", &fields)?,
                    }
                )
            }
            "public.node_tags" => {
                Ok(
                    NodeTags {
                        node_id: Self::index("node_id", &fields)?,
                        version: Self::index("version", &fields)?,
                        k: Self::index("k", &fields)?,
                        v: Self::index("v", &fields)?,
                    }
                )
            }
            "public.ways" => {
                Ok(
                    Ways {
                        way_id: Self::index("way_id", &fields)?,
                        changeset_id: Self::index("changeset_id", &fields)?,
                        timestamp: Self::index("\"timestamp\"", &fields)?,
                        version: Self::index("version", &fields)?,
                        visible: Self::index("visible", &fields)?,
                        redaction_id: Self::index("redaction_id", &fields)?,
                    }
                )
            }
            "public.way_nodes" => {
                Ok(
                    WayNodes {
                        way_id: Self::index("way_id", &fields)?,
                        node_id: Self::index("node_id", &fields)?,
                        version: Self::index("version", &fields)?,
                        sequence_id: Self::index("sequence_id", &fields)?,
                    }
                )
            }
            "public.way_tags" => {
                Ok(
                    WayTags {
                        way_id: Self::index("way_id", &fields)?,
                        k: Self::index("k", &fields)?,
                        v: Self::index("v", &fields)?,
                        version: Self::index("version", &fields)?,
                    }
                )
            }
            "public.relations" => {
                Ok(
                    Relations {
                        relation_id: Self::index("relation_id", &fields)?,
                        changeset_id: Self::index("changeset_id", &fields)?,
                        timestamp: Self::index("\"timestamp\"", &fields)?,
                        version: Self::index("version", &fields)?,
                        visible: Self::index("visible", &fields)?,
                        redaction_id: Self::index("redaction_id", &fields)?,
                    }
                )
            }
            "public.relation_members" => {
                Ok(
                    RelationMembers {
                        relation_id: Self::index("relation_id", &fields)?,
                        member_type: Self::index("member_type", &fields)?,
                        member_id: Self::index("member_id", &fields)?,
                        member_role: Self::index("member_role", &fields)?,
                        version: Self::index("version", &fields)?,
                        sequence_id: Self::index("sequence_id", &fields)?,
                    }
                )
            }
            "public.relation_tags" => {
                Ok(
                    RelationTags {
                        relation_id: Self::index("relation_id", &fields)?,
                        k: Self::index("k", &fields)?,
                        v: Self::index("v", &fields)?,
                        version: Self::index("version", &fields)?,
                    }
                )
            }
            "public.changesets" => {
                Ok(
                    Changesets {
                        id: Self::index("id", &fields)?,
                        user_id: Self::index("user_id", &fields)?,
                        created_at: Self::index("created_at", &fields)?,
                        min_lat: Self::index("min_lat", &fields)?,
                        max_lat: Self::index("max_lat", &fields)?,
                        min_lon: Self::index("min_lon", &fields)?,
                        max_lon: Self::index("max_lon", &fields)?,
                        closed_at: Self::index("closed_at", &fields)?,
                        num_changes: Self::index("num_changes", &fields)?,
                    }
                )
            }
            "public.users" => {
                Ok(
                    Users {
                        email: Self::index("email", &fields)?,
                        id: Self::index("id", &fields)?,
                        pass_crypt: Self::index("pass_crypt", &fields)?,
                        creation_time: Self::index("creation_time", &fields)?,
                        display_name: Self::index("display_name", &fields)?,
                        data_public: Self::index("data_public", &fields)?,
                        description: Self::index("description", &fields)?,
                        home_lat: Self::index("home_lat", &fields)?,
                        home_lon: Self::index("home_lon", &fields)?,
                        home_zoom: Self::index("home_zoom", &fields)?,
                        pass_salt: Self::index("pass_salt", &fields)?,
                        email_valid: Self::index("email_valid", &fields)?,
                        new_email: Self::index("new_email", &fields)?,
                        creation_ip: Self::index("creation_ip", &fields)?,
                        languages: Self::index("languages", &fields)?,
                        status: Self::index("status", &fields)?,
                        terms_agreed: Self::index("terms_agreed", &fields)?,
                        consider_pd: Self::index("consider_pd", &fields)?,
                        auth_uid: Self::index("auth_uid", &fields)?,
                        preferred_editor: Self::index("preferred_editor", &fields)?,
                        terms_seen: Self::index("terms_seen", &fields)?,
                        description_format: Self::index("description_format", &fields)?,
                        changesets_count: Self::index("changesets_count", &fields)?,
                        traces_count: Self::index("traces_count", &fields)?,
                        diary_entries_count: Self::index("diary_entries_count", &fields)?,
                        image_use_gravatar: Self::index("image_use_gravatar", &fields)?,
                        auth_provider: Self::index("auth_provider", &fields)?,
                        home_tile: Self::index("home_tile", &fields)?,
                        tou_agreed: Self::index("tou_agreed", &fields)?,
                    }
                )
            }
            _ => {
                Err(anyhow!("Unknown table: {}", name))
            }
        }
    }
}
