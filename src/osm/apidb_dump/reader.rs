use std::collections::HashMap;
use std::fs;
use std::ops::{AddAssign, SubAssign};
use std::path::PathBuf;
use regex::Regex;
use transient_btree_index::{BtreeConfig, BtreeIndex};
use crate::error::GenericError;
use crate::osm::apidb_dump::block_iterator::BlockIterator;
use crate::osm::apidb_dump::table_def::TableDef;

pub struct Reader {
    input_path: PathBuf,
    tmp_path: PathBuf,
    tables: HashMap<String, TableDef>,
}

impl Reader {
    pub fn new(input_path: PathBuf, tmp_path: PathBuf) -> Result<Reader, GenericError> {
        let mut tables: HashMap<String, TableDef> = HashMap::new();

        let toc_path = input_path.join("toc.dat");
        let toc = fs::read(toc_path)?;
        let raw_table_defs = Self::get_table_def_strings(&toc);
        // COPY public.node_tags (node_id, version, k, v) FROM stdin
        let re = Regex::new("^([^ ]+) \\((.+)\\)$").unwrap();
        for raw_table_def in raw_table_defs {
            let table_data_path = input_path.join(&raw_table_def.1);
            let captures = re.captures(&raw_table_def.0).unwrap();
            let name = captures.get(1).unwrap().as_str();
            let fields: Vec<&str> = captures.get(2).unwrap().as_str().split(", ").collect();
            tables.insert(
                name.to_string(),
                TableDef::new(
                    name.to_string(),
                    table_data_path,
                    tmp_path.clone(),
                    fields.iter().map(|e| {
                        e.to_string()
                    }
                    ).collect(),
                )?,
            );
        }

        Ok(
            Reader {
                input_path,
                tmp_path,
                tables,
            }
        )
    }

    fn get_table_def_strings(toc: &Vec<u8>) -> Vec<(String, String)> {

        // COPY public.node_tags (node_id, version, k, v) FROM stdin;......3838.dat
        let mut result: Vec<(String, String)> = Vec::new();
        let copy = "COPY ".as_bytes();
        let from_stdin = " FROM stdin".as_bytes();
        let dotdat = ".dat".as_bytes();
        let mut i: usize = 0;
        let mut start_table_def;
        let mut end_table_def;
        let mut start_file_name;
        let mut end_file_name;
        while i < toc.len() {
            if toc[i..].starts_with(copy) {
                i.add_assign(copy.len());
                start_table_def = i;
                while i < toc.len() {
                    if toc[i..].starts_with(from_stdin) {
                        end_table_def = i;
                        i.add_assign(from_stdin.len());
                        while i < toc.len() {
                            if toc[i..].starts_with(dotdat) {
                                start_file_name = i - 1;
                                i.add_assign(dotdat.len());
                                end_file_name = i;
                                while start_file_name > 0 && toc[start_file_name].is_ascii_digit() {
                                    start_file_name.sub_assign(1);
                                }
                                start_file_name.add_assign(1);
                                result.push(
                                    (
                                        String::from_utf8(toc[start_table_def..end_table_def].to_vec()).unwrap(),
                                        String::from_utf8(toc[start_file_name..end_file_name].to_vec()).unwrap(),
                                    )
                                );
                                break;
                            }
                            i.add_assign(1);
                        }
                        break;
                    }
                    i.add_assign(1);
                }
            }
            i.add_assign(1);
        }
        result
    }

    pub fn blocks(&self) -> Result<BlockIterator, GenericError> {
        BlockIterator::new(self.tables.clone())
    }
}



