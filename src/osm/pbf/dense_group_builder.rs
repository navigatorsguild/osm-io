use crate::osm::model::node::Node;
use crate::osm::pbf::string_table_builder::StringTableBuilder;
use crate::osmpbf::{DenseInfo, DenseNodes, PrimitiveGroup};

pub(crate) struct DenseGroupBuilder {
    granularity: i32,
    date_granularity: i32,
    lat_offset: i64,
    lon_offset: i64,
    dense: Option<DenseNodes>,
    last_id: i64,
    last_lon: i64,
    last_lat: i64,
    last_timestamp: i64,
    last_uid: i32,
    last_changeset: i64,
    last_sid: i32,
}

impl DenseGroupBuilder {
    pub(crate) fn new(
        granularity: i32,
        date_granularity: i32,
        lat_offset: i64,
        lon_offset: i64,
        node: &Node,
        string_table_builder: &mut StringTableBuilder,
    ) -> DenseGroupBuilder {
        let mut dense = Some(DenseNodes::default());
        let last_id;
        let last_lon;
        let last_lat;
        let last_timestamp;
        let last_uid;
        let last_changeset;
        let last_sid;

        last_id = node.id();
        dense.as_mut().unwrap().id.push(last_id);

        last_lon = (node.coordinate().lon() * 1000000000 as f64  / granularity as f64 - lon_offset as f64) as i64;
        dense.as_mut().unwrap().lon.push(last_lon);
        last_lat = (node.coordinate().lat() * 1000000000  as f64 / granularity  as f64 - lat_offset as f64) as i64;
        dense.as_mut().unwrap().lat.push(last_lat);

        dense.as_mut().unwrap().denseinfo = Some(DenseInfo::default());
        dense.as_mut().unwrap().denseinfo.as_mut().unwrap().visible.push(node.visible());

        last_timestamp = node.timestamp() / date_granularity as i64;
        dense.as_mut().unwrap().denseinfo.as_mut().unwrap().timestamp.push(last_timestamp);

        dense.as_mut().unwrap().denseinfo.as_mut().unwrap().version.push(node.version());

        last_uid = node.uid();
        dense.as_mut().unwrap().denseinfo.as_mut().unwrap().uid.push(last_uid);

        last_changeset = node.changeset();
        dense.as_mut().unwrap().denseinfo.as_mut().unwrap().changeset.push(last_changeset);

        last_sid = string_table_builder.add(node.user());
        dense.as_mut().unwrap().denseinfo.as_mut().unwrap().user_sid.push(last_sid);

        dense.as_mut().unwrap().keys_vals.push(0);

        DenseGroupBuilder {
            granularity,
            date_granularity,
            lat_offset,
            lon_offset,
            dense,
            last_id,
            last_lon,
            last_lat,
            last_timestamp,
            last_uid,
            last_changeset,
            last_sid,
        }
    }

    pub(crate) fn add(&mut self, node: &Node, string_table_builder: &mut StringTableBuilder) {
        let current_id = node.id();
        self.dense.as_mut().unwrap().id.push(current_id - self.last_id);
        self.last_id = current_id;

        let current_lon = (node.coordinate().lon() * 1000000000 as f64  / self.granularity as f64 - self.lon_offset as f64) as i64;
        self.dense.as_mut().unwrap().lon.push(current_lon - self.last_lon);
        self.last_lon = current_lon;
        let current_lat = (node.coordinate().lat() * 1000000000  as f64 / self.granularity  as f64 - self.lat_offset as f64) as i64;
        self.dense.as_mut().unwrap().lat.push(current_lat - self.last_lat);
        self.last_lat = current_lat;

        self.dense.as_mut().unwrap().denseinfo.as_mut().unwrap().visible.push(node.visible());

        let current_timestamp = node.timestamp() / self.date_granularity as i64;
        self.dense.as_mut().unwrap().denseinfo.as_mut().unwrap().timestamp.push(current_timestamp - self.last_timestamp);
        self.last_timestamp = current_timestamp;

        self.dense.as_mut().unwrap().denseinfo.as_mut().unwrap().version.push(node.version());

        let current_uid = node.uid();
        self.dense.as_mut().unwrap().denseinfo.as_mut().unwrap().uid.push(current_uid - self.last_uid);
        self.last_uid = current_uid;

        let current_changeset = node.changeset();
        self.dense.as_mut().unwrap().denseinfo.as_mut().unwrap().changeset.push(current_changeset - self.last_changeset);
        self.last_changeset = current_changeset;

        let current_sid = string_table_builder.add(node.user());
        self.dense.as_mut().unwrap().denseinfo.as_mut().unwrap().user_sid.push(current_sid - self.last_sid);
        self.last_sid = current_sid;

        for tag in node.tags() {
            let key_index = string_table_builder.add(tag.k());
            let value_index = string_table_builder.add(tag.v());
            self.dense.as_mut().unwrap().keys_vals.push(key_index);
            self.dense.as_mut().unwrap().keys_vals.push(value_index);
        }
        self.dense.as_mut().unwrap().keys_vals.push(0);
    }

    pub(crate) fn build(&mut self) -> PrimitiveGroup {
        let mut primitive_group = PrimitiveGroup::default();
        primitive_group.dense = self.dense.replace(DenseNodes::default());
        primitive_group
    }
}