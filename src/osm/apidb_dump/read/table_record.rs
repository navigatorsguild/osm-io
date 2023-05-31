use crate::osm::apidb_dump::read::changeset_record::ChangesetRecord;
use crate::osm::apidb_dump::read::node_record::NodeRecord;
use crate::osm::apidb_dump::read::node_tag_record::NodeTagRecord;
use crate::osm::apidb_dump::read::relation_member_record::RelationMemberRecord;
use crate::osm::apidb_dump::read::relation_record::RelationRecord;
use crate::osm::apidb_dump::read::relation_tag_record::RelationTagRecord;
use crate::osm::apidb_dump::read::user_record::UserRecord;
use crate::osm::apidb_dump::read::way_node_record::WayNodeRecord;
use crate::osm::apidb_dump::read::way_record::WayRecord;
use crate::osm::apidb_dump::read::way_tag_record::WayTagRecord;

#[derive(Debug)]
pub(crate) enum TableRecord {
    Node {
        node_record: NodeRecord,
    },
    NodeTag {
        node_tag_record: NodeTagRecord,
    },
    Way {
        way_record: WayRecord,
    },
    WayTag {
        way_tag_record: WayTagRecord,
    },
    WayNode {
        way_node_record: WayNodeRecord,
    },
    Relation {
        relation_record: RelationRecord,
    },
    RelationTag {
        relation_tag_record: RelationTagRecord,
    },
    RelationMember {
        relation_member_record: RelationMemberRecord,
    },
    Changeset {
        changeset_record: ChangesetRecord,
    },
    User {
        user_record: UserRecord,
    },
}