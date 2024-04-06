use std::fs;
use std::path::PathBuf;
use anyhow::Context;
use json::JsonValue;

static TOC: &[u8] = include_bytes!("./toc/toc.dat");
static MAPPING: &str = include_str!("./toc/mapping.json");

pub(crate) fn write_toc(path: &PathBuf) -> Result<(), anyhow::Error> {
    let toc_path = path.join(PathBuf::from("toc.dat"));
    fs::write(&toc_path, TOC).with_context(|| format!("write {:?} to {:?}", &toc_path, path))?;
    Ok(())
}

pub(crate) fn load_template_mapping() -> Result<JsonValue, anyhow::Error> {
    Ok(json::parse(MAPPING)?)
}

