#[derive(Debug)]
pub(crate) struct RelationTagRecord {
    relation_id: i64,
    version: i64,
    k: String,
    v: String,
}

impl RelationTagRecord {
    pub(crate) fn new(
        relation_id: i64,
        version: i64,
        k: String,
        v: String,
    ) -> RelationTagRecord {
        RelationTagRecord {
            relation_id,
            version,
            k,
            v,
        }
    }

    pub(crate) fn relation_id(&self) -> i64 {
        self.relation_id
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
