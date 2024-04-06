use std::borrow::BorrowMut;
use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;

pub(crate) struct TableDataWriter {
    table_name: String,
    file_path: PathBuf,
    writer: BufWriter<File>,
}

impl TableDataWriter {
    pub(crate) fn new(table_name: String, file_name: String, output_path: &Path) -> Result<TableDataWriter, anyhow::Error> {
        let file_path = output_path.join(file_name);
        let writer = BufWriter::new(File::create(&file_path)?);
        Ok(TableDataWriter {
            table_name,
            file_path,
            writer,
        })
    }

    pub(crate) fn close(&mut self) -> Result<(), anyhow::Error> {
        self.writer.write_all("\\.\n".as_bytes()).with_context(|| format!("Problem writing table data footer: {:?}", self.file_path))?;
        self.writer.flush().with_context(|| format!("Problem flushing table data file {:?}", self.file_path))?;
        Ok(())
    }

    pub(crate) fn writer(&mut self) -> &mut BufWriter<File> {
        self.writer.borrow_mut()
    }

    pub(crate) fn table_name(&self) -> &str {
        &self.table_name
    }

    pub(crate) fn file_path(&self) -> &PathBuf {
        &self.file_path
    }
}

impl Debug for TableDataWriter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "table: {}, file: {}", self.table_name(), self.file_path().display())
    }
}