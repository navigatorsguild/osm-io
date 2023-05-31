#[derive(Debug)]
pub(crate) struct NodeTagRecord {
    node_id: i64,
    version: i64,
    k: String,
    v: String,
}

impl NodeTagRecord {
    pub(crate) fn new(
        node_id: i64,
        version: i64,
        k: String,
        v: String,
    ) -> NodeTagRecord {
        NodeTagRecord {
            node_id,
            version,
            k,
            v,
        }
    }

    pub(crate) fn node_id(&self) -> i64 {
        self.node_id
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
