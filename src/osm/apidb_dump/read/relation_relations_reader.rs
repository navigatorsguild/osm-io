use crate::osm::apidb_dump::read::relation_member_record::RelationMemberRecord;
use crate::osm::apidb_dump::read::relation_relation::RelationRelation;
use crate::osm::apidb_dump::read::relation_tag_record::RelationTagRecord;
use crate::osm::apidb_dump::read::table_def::TableDef;
use crate::osm::apidb_dump::read::table_reader::{TableIterator, TableReader};
use crate::osm::apidb_dump::read::table_record::TableRecord;

#[derive(Clone)]
pub(crate) struct RelationRelationsReader {
    relations_reader: TableReader,
    relation_members_reader: TableReader,
    relation_tags_reader: TableReader,
}

impl RelationRelationsReader {
    pub(crate) fn new(
        relations_def: &TableDef,
        relation_members_def: &TableDef,
        relation_tags_def: &TableDef,
    ) -> Result<RelationRelationsReader, anyhow::Error> {
        let relations_reader = TableReader::new(relations_def)?;
        let relation_members_reader = TableReader::new(relation_members_def)?;
        let relation_tags_reader = TableReader::new(relation_tags_def)?;
        Ok(
            RelationRelationsReader {
                relations_reader,
                relation_members_reader,
                relation_tags_reader,
            }
        )
    }
}

impl IntoIterator for RelationRelationsReader {
    type Item = RelationRelation;
    type IntoIter = RelationRelationsIterator;

    fn into_iter(self) -> Self::IntoIter {
        RelationRelationsIterator::new(&self).unwrap()
    }
}

pub(crate) struct RelationRelationsIterator {
    relations_iterator: TableIterator,
    relation_members_iterator: TableIterator,
    relation_tags_iterator: TableIterator,
    next_relation_member_record: Option<RelationMemberRecord>,
    next_relation_tag_record: Option<RelationTagRecord>,
}


impl RelationRelationsIterator {
    pub(crate) fn new(relation_relations_reader: &RelationRelationsReader) -> Result<RelationRelationsIterator, anyhow::Error> {
        let reader = relation_relations_reader.clone();
        let relations_iterator = reader.relations_reader.clone().into_iter();
        let relation_members_iterator = reader.relation_members_reader.clone().into_iter();
        let relation_tags_iterator = reader.relation_tags_reader.clone().into_iter();
        Ok(
            RelationRelationsIterator {
                relations_iterator,
                relation_members_iterator,
                relation_tags_iterator,
                next_relation_member_record: None,
                next_relation_tag_record: None,
            }
        )
    }
}

impl Iterator for RelationRelationsIterator {
    type Item = RelationRelation;

    fn next(&mut self) -> Option<Self::Item> {
        // ADD CONSTRAINT relation_members_id_fkey FOREIGN KEY (relation_id, version) REFERENCES public.relations(relation_id, version);
        // ADD CONSTRAINT relation_tags_id_fkey FOREIGN KEY (relation_id, version) REFERENCES public.relations(relation_id, version);
        if let Some(relation) = self.relations_iterator.next() {
            if let TableRecord::Relation { relation_record } = relation {
                let mut current_relation_tags = Vec::<RelationTagRecord>::new();
                if let Some(relation_tag_record) = self.next_relation_tag_record.take() {
                    if relation_tag_record.relation_id() == relation_record.relation_id() && relation_tag_record.version() == relation_record.version() {
                        current_relation_tags.push(relation_tag_record);
                        for relation_tag in self.relation_tags_iterator.by_ref() {
                            if let TableRecord::RelationTag { relation_tag_record } = relation_tag {
                                if relation_tag_record.relation_id() == relation_record.relation_id() && relation_tag_record.version() == relation_record.version() {
                                    current_relation_tags.push(relation_tag_record)
                                } else {
                                    self.next_relation_tag_record = Some(relation_tag_record);
                                    break;
                                }
                            } else {
                                panic!("Found incorrect record type, not a TableRecord:RelationTag");
                            }
                        }
                    } else {
                        self.next_relation_tag_record = Some(relation_tag_record);
                    }
                } else {
                    for relation_tag in self.relation_tags_iterator.by_ref() {
                        if let TableRecord::RelationTag { relation_tag_record } = relation_tag {
                            if relation_tag_record.relation_id() == relation_record.relation_id() && relation_tag_record.version() == relation_record.version() {
                                current_relation_tags.push(relation_tag_record)
                            } else {
                                self.next_relation_tag_record = Some(relation_tag_record);
                                break;
                            }
                        } else {
                            panic!("Found incorrect record type, not a TableRecord:RelationTag");
                        }
                    }
                }

                let mut current_relation_members = Vec::<RelationMemberRecord>::new();
                if let Some(relation_member_record) = self.next_relation_member_record.take() {
                    if relation_member_record.relation_id() == relation_record.relation_id() && relation_member_record.version() == relation_record.version() {
                        current_relation_members.push(relation_member_record);
                        for relation_member in self.relation_members_iterator.by_ref() {
                            if let TableRecord::RelationMember { relation_member_record } = relation_member {
                                if relation_member_record.relation_id() == relation_record.relation_id() && relation_member_record.version() == relation_record.version() {
                                    current_relation_members.push(relation_member_record)
                                } else {
                                    self.next_relation_member_record = Some(relation_member_record);
                                    break;
                                }
                            } else {
                                panic!("Found incorrect record type, not a TableRecord:RelationMember");
                            }
                        }
                    } else {
                        self.next_relation_member_record = Some(relation_member_record);
                    }
                } else {
                    for relation_member in self.relation_members_iterator.by_ref() {
                        if let TableRecord::RelationMember { relation_member_record } = relation_member {
                            if relation_member_record.relation_id() == relation_record.relation_id() && relation_member_record.version() == relation_record.version() {
                                current_relation_members.push(relation_member_record)
                            } else {
                                self.next_relation_member_record = Some(relation_member_record);
                                break;
                            }
                        } else {
                            panic!("Found incorrect record type, not a TableRecord:RelationMember");
                        }
                    }
                }

                Some(
                    RelationRelation::new(
                        relation_record,
                        current_relation_members,
                        current_relation_tags,
                    )
                )
            } else {
                panic!("Found incorrect record type, not a TableRecord:Relation");
            }
        } else {
            None
        }
    }
}
