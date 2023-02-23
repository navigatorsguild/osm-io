use crate::osm::model::relation::{Member, Relation};
use crate::osm::pbf::string_table_builder::StringTableBuilder;
use crate::osmpbf;
use crate::osmpbf::PrimitiveGroup;

pub(crate) struct RelationsGroupBuilder {
    relations: Option<Vec<osmpbf::Relation>>,
}

impl RelationsGroupBuilder {
    pub(crate) fn new(date_granularity: i32, relation: &Relation, string_table_builder: &mut StringTableBuilder) -> RelationsGroupBuilder {
        let mut relations = Some(Vec::<osmpbf::Relation>::with_capacity(8000));
        relations.as_mut().unwrap().push(Self::convert(relation, string_table_builder));

        RelationsGroupBuilder {
            relations,
        }
    }

    pub(crate) fn add(&mut self, relation: &Relation, string_table_builder: &mut StringTableBuilder) {
        self.relations.as_mut().unwrap().push(Self::convert(relation, string_table_builder));
    }

    fn convert(relation: &Relation, string_table_builder: &mut StringTableBuilder) -> osmpbf::Relation {
        let mut r = osmpbf::Relation::default();

        r.id = relation.id();
        let mut last_memid = 0;
        for member in relation.members() {
            let mut current_memid = 0;
            match member {
                Member::Node { member } => {
                    current_memid = member.id();
                    r.types.push(osmpbf::relation::MemberType::Node as i32);
                    r.roles_sid.push(string_table_builder.add(member.role()))
                }
                Member::Way { member } => {
                    current_memid = member.id();
                    r.types.push(osmpbf::relation::MemberType::Way as i32);
                    r.roles_sid.push(string_table_builder.add(member.role()))
                }
                Member::Relation { member } => {
                    current_memid = member.id();
                    r.types.push(osmpbf::relation::MemberType::Relation as i32);
                    r.roles_sid.push(string_table_builder.add(member.role()))
                }
            }
            r.memids.push(current_memid - last_memid);
            last_memid = current_memid;
        }

        for tag in relation.tags() {
            let key_index = string_table_builder.add(&tag.k);
            let val_index = string_table_builder.add(&tag.v);
            r.keys.push(key_index as u32);
            r.vals.push(val_index as u32)
        }

        r.info = Some(osmpbf::Info::default());
        r.info.as_mut().unwrap().visible = Some(relation.visible());
        r.info.as_mut().unwrap().uid = Some(relation.uid());
        r.info.as_mut().unwrap().changeset = Some(relation.changeset());
        r.info.as_mut().unwrap().timestamp = Some(relation.timestamp());
        r.info.as_mut().unwrap().version = Some(relation.version());
        r.info.as_mut().unwrap().user_sid = Some(string_table_builder.add(relation.user()) as u32);

        r
    }

    pub(crate) fn build(&mut self) -> PrimitiveGroup {
        let mut primitive_group = PrimitiveGroup::default();
        primitive_group.relations = self.relations.replace(Vec::<osmpbf::Relation>::new()).unwrap();
        primitive_group
    }
}