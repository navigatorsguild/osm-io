use chrono::format::Numeric::Timestamp;
use crate::osm::model::way::Way;
use crate::osm::pbf::string_table_builder::StringTableBuilder;
use crate::osmpbf;
use crate::osmpbf::{Info, PrimitiveGroup};

pub(crate) struct WaysGroupBuilder {
    ways: Option<Vec<osmpbf::Way>>,
}

impl WaysGroupBuilder {
    pub(crate) fn new(date_granularity: i32, way: &Way, string_table_builder: &mut StringTableBuilder) -> WaysGroupBuilder {
        let mut ways = Some(Vec::<osmpbf::Way>::with_capacity(8000));
        ways.as_mut().unwrap().push(Self::convert(way, string_table_builder));

        WaysGroupBuilder {
            ways,
        }
    }

    pub(crate) fn add(&mut self, way: &Way, string_table_builder: &mut StringTableBuilder) {
        self.ways.as_mut().unwrap().push(Self::convert(way, string_table_builder));
    }

    fn convert(way: &Way, string_table_builder: &mut StringTableBuilder) -> osmpbf::Way {
        let mut w = osmpbf::Way::default();
        w.id = way.id();
        let mut last_ref = 0;
        for r in way.refs() {
            w.refs.push(r - last_ref);
            last_ref = *r;
        }

        for tag in way.tags() {
            let key_index = string_table_builder.add(&tag.k);
            let val_index = string_table_builder.add(&tag.v);
            w.keys.push(key_index as u32);
            w.vals.push(val_index as u32)
        }
        w.info = Some(osmpbf::Info::default());
        w.info.as_mut().unwrap().visible = Some(way.visible());
        w.info.as_mut().unwrap().uid = Some(way.uid());
        w.info.as_mut().unwrap().changeset = Some(way.changeset());
        w.info.as_mut().unwrap().timestamp = Some(way.timestamp());
        w.info.as_mut().unwrap().version = Some(way.version());
        w.info.as_mut().unwrap().user_sid = Some(string_table_builder.add(way.user()) as u32);
        w
    }

    pub(crate) fn build(&mut self) -> PrimitiveGroup {
        let mut primitive_group = PrimitiveGroup::default();
        primitive_group.ways = self.ways.replace(Vec::<osmpbf::Way>::new()).unwrap();
        primitive_group
    }
}

