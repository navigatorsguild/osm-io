use std::path::PathBuf;

use crate::error::GenericError;
use crate::osm::apidb_dump::table_fields::TableFields;

#[derive(Debug, Clone)]
pub struct TableDef {
    name: String,
    path: PathBuf,
    fields: TableFields,
}

impl TableDef {
    pub fn new(name: String, path: PathBuf, fields: Vec<String>) -> Result<TableDef, GenericError> {
        let table_def = TableDef {
            name: name.clone(),
            path,
            fields: TableFields::new(name.clone(), fields)?,
        };
        Ok(table_def)
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn path(&self) -> PathBuf{
        self.path.clone()
    }

    pub fn fields(&self) -> TableFields {
        self.fields.clone()
    }
}
