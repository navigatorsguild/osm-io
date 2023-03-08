use crate::osm::apidb_dump::relation_member_record::RelationMemberRecord;
use crate::osm::apidb_dump::relation_record::RelationRecord;
use crate::osm::apidb_dump::relation_tag_record::RelationTagRecord;

#[derive(Debug)]
pub(crate) struct RelationRelation{
    relation: RelationRecord,
    relation_members: Vec<RelationMemberRecord>,
    tags: Vec<RelationTagRecord>,
}


impl RelationRelation {
    pub(crate)  fn new(
        relation: RelationRecord,
        relation_members: Vec<RelationMemberRecord>,
        tags: Vec<RelationTagRecord>,
    ) -> RelationRelation {
        RelationRelation {
            relation,
            relation_members,
            tags,
        }
    }

    pub(crate) fn relation(&self) -> &RelationRecord {
        &self.relation
    }

    #[allow(dead_code)]
    pub(crate) fn relation_members(&self) -> &Vec<RelationMemberRecord> {
        &self.relation_members
    }

    pub(crate) fn take_relation_members(&mut self) -> Vec<RelationMemberRecord> {
        std::mem::take(&mut self.relation_members)
    }

    #[allow(dead_code)]
    pub(crate) fn tags(&self) -> &Vec<RelationTagRecord> {
        &self.tags
    }

    pub(crate) fn take_tags(&mut self) -> Vec<RelationTagRecord> {
        std::mem::take(&mut self.tags)
    }
}