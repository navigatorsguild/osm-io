use crate::osm::model::tag::Tag;

#[derive(Debug)]
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
}

#[derive(Debug)]
pub enum Member {
    Node {
        member: MemberData,
    },
    Way {
        member: MemberData,
    },
    Relation {
        member: MemberData,
    }
}


#[derive(Debug)]
pub struct Relation {
   id: i64,
    timestamp: i64,
    changeset: i64,
    uid: i32,
    user: String,
    visible: bool,
    members: Vec<Member>,
    tags: Vec<Tag>,
}

impl Relation {
    pub fn new(id: i64, timestamp: i64, changeset: i64, uid: i32, user: String, visible: bool, members: Vec<Member>, tags: Vec<Tag>) -> Relation {
       Relation {
           id,
           timestamp,
           changeset,
           uid,
           user,
           visible,
           members,
           tags,
       }
    }
}