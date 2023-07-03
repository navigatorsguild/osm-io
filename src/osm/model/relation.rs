use crate::osm::model::tag::Tag;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MemberData {
    id: i64,
    role: String,
}

impl MemberData {
    pub fn new(id: i64, role: String) -> MemberData {
        MemberData {
            id,
            role,
        }
    }
    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn role(&self) -> &String {
        &self.role
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Member {
    Node {
        member: MemberData,
    },
    Way {
        member: MemberData,
    },
    Relation {
        member: MemberData,
    },
}

#[derive(Debug, Clone)]
pub struct Relation {
    id: i64,
    version: i32,
    timestamp: i64,
    changeset: i64,
    uid: i32,
    user: String,
    visible: bool,
    members: Vec<Member>,
    tags: Vec<Tag>,
}

impl Relation {
    pub fn new(id: i64, version: i32, timestamp: i64, changeset: i64, uid: i32, user: String, visible: bool, members: Vec<Member>, tags: Vec<Tag>) -> Relation {
        Relation {
            id,
            version,
            timestamp,
            changeset,
            uid,
            user,
            visible,
            members,
            tags,
        }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn changeset(&self) -> i64 {
        self.changeset
    }

    pub fn uid(&self) -> i32 {
        self.uid
    }

    pub fn user(&self) -> &String {
        &self.user
    }

    pub fn take_user(&mut self) -> String {
        std::mem::take(&mut self.user)
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn members(&self) -> &Vec<Member> {
        &self.members
    }

    pub fn tags(&self) -> &Vec<Tag> {
        &self.tags
    }

    pub fn take_tags(&mut self) -> Vec<Tag> {
        std::mem::take(&mut self.tags)
    }
}