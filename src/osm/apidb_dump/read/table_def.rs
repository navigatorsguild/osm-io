use std::path::PathBuf;

use crate::osm::apidb_dump::read::table_fields::TableFields;
use crate::osm::apidb_dump::read::table_pkey::TablePkey;

#[derive(Debug, Clone)]
pub(crate) struct TableDef {
    name: String,
    path: PathBuf,
    sorted_path: PathBuf,
    tmp_path: PathBuf,
    fields: TableFields,
    pkey: TablePkey,
}

impl TableDef {
    fn build_sorted_path(name: String, mut path: PathBuf) -> PathBuf {
        path.push(PathBuf::from(format!("sorted-{}.dat", name)));
        path
    }

    pub(crate) fn new(name: String, path: PathBuf, tmp_path: PathBuf, fields: Vec<String>) -> Result<TableDef, anyhow::Error> {
        let table_def = TableDef {
            name: name.clone(),
            path: path.clone(),
            sorted_path: Self::build_sorted_path(name.clone(), tmp_path.clone()),
            tmp_path,
            fields: TableFields::new(name.clone(), fields.clone())?,
            pkey: TablePkey::new(name.clone(), fields.clone())?,
        };
        Ok(table_def)
    }

    pub(crate) fn name(&self) -> String {
        self.name.clone()
    }

    pub(crate) fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub(crate) fn sorted_path(&self) -> PathBuf {
        self.sorted_path.clone()
    }

    pub(crate) fn tmp_path(&self) -> PathBuf {
        self.tmp_path.clone()
    }

    #[allow(dead_code)]
    pub(crate) fn fields(&self) -> TableFields {
        self.fields.clone()
    }

    pub(crate) fn fields_ref(&self) -> &TableFields {
        &self.fields
    }

    pub(crate) fn pkey(&self) -> TablePkey {
        self.pkey.clone()
    }
}
