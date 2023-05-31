use anyhow::anyhow;

#[derive(Debug, Copy, Clone)]
pub(crate) enum RelationMemberType {
    Node,
    Way,
    Relation,
}

impl TryFrom<&str> for RelationMemberType {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "n" => {
                Ok(RelationMemberType::Node)
            },
            "Node" => {
                Ok(RelationMemberType::Node)
            },
            "w" => {
                Ok(RelationMemberType::Way)
            },
            "Way" => {
                Ok(RelationMemberType::Way)
            },
            "r" => {
                Ok(RelationMemberType::Relation)
            },
            "Relation" => {
                Ok(RelationMemberType::Relation)
            },
            _ => {
                Err(anyhow!("Unknown relation member type: {}", value))
            },
        }
    }
}

#[derive(Debug)]
pub(crate) struct RelationMemberRecord {
    relation_id: i64,
    member_type: RelationMemberType,
    member_id: i64,
    member_role: String,
    version: i64,
    sequence_id: i64,
}

impl RelationMemberRecord {
    pub(crate) fn new(
        relation_id: i64,
        member_type: RelationMemberType,
        member_id: i64,
        member_role: String,
        version: i64,
        sequence_id: i64,
    ) -> RelationMemberRecord {
        RelationMemberRecord {
            relation_id,
            member_type,
            member_id,
            member_role,
            version,
            sequence_id,
        }
    }

    pub(crate) fn relation_id(&self) -> i64 {
        self.relation_id
    }

    pub(crate) fn member_type(&self) -> RelationMemberType {
        self.member_type
    }

    pub(crate) fn member_id(&self) -> i64 {
        self.member_id
    }

    #[allow(dead_code)]
    pub(crate) fn member_role(&self) -> &String {
        &self.member_role
    }

    pub(crate) fn take_member_role(&mut self) -> String {
        std::mem::take(&mut self.member_role)
    }

    pub(crate) fn version(&self) -> i64 {
        self.version
    }

    #[allow(dead_code)]
    pub(crate) fn sequence_id(&self) -> i64 {
        self.sequence_id
    }
}