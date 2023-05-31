use anyhow::anyhow;
use text_file_sort::field::Field;
use text_file_sort::field_type::FieldType;

#[derive(Clone, Debug)]
pub struct TablePkey {
    name: String,
    key: Vec<Field>,
}

impl TablePkey {
    pub fn new(name: String, fields: Vec<String>) -> Result<TablePkey, anyhow::Error> {
        let mut key = Vec::new();
        let mut error = None;
        match name.as_str() {
            "public.nodes" => {
                // ADD CONSTRAINT nodes_pkey PRIMARY KEY (node_id, version);
                key.push(Field::new(Self::index("node_id", &fields)?, FieldType::Integer).with_str_name("node_id"));
                key.push(Field::new(Self::index("version", &fields)?, FieldType::Integer).with_str_name("version"));
            }
            "public.node_tags" => {
                // ADD CONSTRAINT node_tags_pkey PRIMARY KEY (node_id, version, k);
                key.push(Field::new(Self::index("node_id", &fields)?, FieldType::Integer).with_str_name("node_id"));
                key.push(Field::new(Self::index("version", &fields)?, FieldType::Integer).with_str_name("version"));
                key.push(Field::new(Self::index("k", &fields)?, FieldType::String).with_str_name("k"));
            }
            "public.ways" => {
                // ADD CONSTRAINT ways_pkey PRIMARY KEY (way_id, version);
                key.push(Field::new(Self::index("way_id", &fields)?, FieldType::Integer).with_str_name("way_id"));
                key.push(Field::new(Self::index("version", &fields)?, FieldType::Integer).with_str_name("version"));
            }
            "public.way_nodes" => {
                // ADD CONSTRAINT way_nodes_pkey PRIMARY KEY (way_id, version, sequence_id);
                key.push(Field::new(Self::index("way_id", &fields)?, FieldType::Integer).with_str_name("way_id"));
                key.push(Field::new(Self::index("version", &fields)?, FieldType::Integer).with_str_name("version"));
                key.push(Field::new(Self::index("sequence_id", &fields)?, FieldType::Integer).with_str_name("sequence_id"));
            }
            "public.way_tags" => {
                // ADD CONSTRAINT way_tags_pkey PRIMARY KEY (way_id, version, k);
                key.push(Field::new(Self::index("way_id", &fields)?, FieldType::Integer).with_str_name("way_id"));
                key.push(Field::new(Self::index("version", &fields)?, FieldType::Integer).with_str_name("version"));
                key.push(Field::new(Self::index("k", &fields)?, FieldType::String).with_str_name("k"));
            }
            "public.relations" => {
                // ADD CONSTRAINT relations_pkey PRIMARY KEY (relation_id, version);
                key.push(Field::new(Self::index("relation_id", &fields)?, FieldType::Integer).with_str_name("relation_id"));
                key.push(Field::new(Self::index("version", &fields)?, FieldType::Integer).with_str_name("version"));
            }
            "public.relation_members" => {
                // ADD CONSTRAINT relation_members_pkey PRIMARY KEY (relation_id, version, member_type, member_id, member_role, sequence_id);
                key.push(Field::new(Self::index("relation_id", &fields)?, FieldType::Integer).with_str_name("relation_id"));
                key.push(Field::new(Self::index("version", &fields)?, FieldType::Integer).with_str_name("version"));
                key.push(Field::new(Self::index("member_type", &fields)?, FieldType::String).with_str_name("member_type"));
                key.push(Field::new(Self::index("member_id", &fields)?, FieldType::Integer).with_str_name("member_id"));
                key.push(Field::new(Self::index("member_role", &fields)?, FieldType::String).with_str_name("member_role"));
                key.push(Field::new(Self::index("sequence_id", &fields)?, FieldType::Integer).with_str_name("sequence_id"));
            }
            "public.relation_tags" => {
                // ADD CONSTRAINT relation_tags_pkey PRIMARY KEY (relation_id, version, k);
                key.push(Field::new(Self::index("relation_id", &fields)?, FieldType::Integer).with_str_name("relation_id"));
                key.push(Field::new(Self::index("version", &fields)?, FieldType::Integer).with_str_name("version"));
                key.push(Field::new(Self::index("k", &fields)?, FieldType::String).with_str_name("k"));
            }
            "public.changesets" => {
                // ADD CONSTRAINT changesets_pkey PRIMARY KEY (id);
                key.push(Field::new(Self::index("id", &fields)?, FieldType::Integer).with_str_name("id"));
            }
            "public.users" => {
                // ADD CONSTRAINT users_pkey PRIMARY KEY (id);
                key.push(Field::new(Self::index("id", &fields)?, FieldType::Integer).with_str_name("id"));
            }
            _ => {
                error = Some(anyhow!("No primary key defined for table: {}", name));
            }
        }
        if error.is_some() {
            Err(error.unwrap())
        } else {
            Ok(
                TablePkey {
                    name,
                    key,
                }
            )
        }
    }

    fn index(v: &str, fields: &Vec<String>) -> Result<usize, anyhow::Error> {
        match fields.iter().position(|e| { *e == v.to_string() }) {
            None => {
                Err(anyhow!("Field not found: {}", v))
            }
            Some(i) => {
                Ok(i + 1)
            }
        }
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn key(&self) -> Vec<Field> {
        self.key.clone()
    }
}