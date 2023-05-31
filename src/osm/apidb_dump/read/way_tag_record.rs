#[derive(Debug)]
pub(crate) struct WayTagRecord {
    way_id: i64,
    version: i64,
    k: String,
    v: String,
}

impl WayTagRecord {
    pub(crate) fn new(
        way_id: i64,
        version: i64,
        k: String,
        v: String,
    ) -> WayTagRecord {
        WayTagRecord {
            way_id,
            version,
            k,
            v,
        }
    }

    pub(crate) fn way_id(&self) -> i64 {
        self.way_id
    }

    pub(crate) fn version(&self) -> i64 {
        self.version
    }

    #[allow(dead_code)]
    pub(crate) fn k(&self) -> &String {
        &self.k
    }

    #[allow(dead_code)]
    pub(crate) fn v(&self) -> &String {
        &self.v
    }

    pub(crate) fn take_k(&mut self) -> String {
        std::mem::take(&mut self.k)
    }

    pub(crate) fn take_v(&mut self) -> String {
        std::mem::take(&mut self.v)
    }
}
