use chrono::{NaiveDateTime};
use crate::error::{GenericError, OsmIoError};
use crate::osm::apidb_dump::changeset_record::ChangesetRecord;
use crate::osm::apidb_dump::node_record::NodeRecord;
use crate::osm::apidb_dump::node_tag_record::NodeTagRecord;
use crate::osm::apidb_dump::relation_member_record::RelationMemberRecord;
use crate::osm::apidb_dump::relation_record::RelationRecord;
use crate::osm::apidb_dump::relation_tag_record::RelationTagRecord;
use crate::osm::apidb_dump::user_record::UserRecord;
use crate::osm::apidb_dump::way_node_record::WayNodeRecord;
use crate::osm::apidb_dump::way_record::WayRecord;
use crate::osm::apidb_dump::way_tag_record::WayTagRecord;

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