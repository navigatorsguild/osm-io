use std::collections::HashMap;
use std::fs;
use std::ops::{AddAssign, SubAssign};
use std::path::PathBuf;
use anyhow::{anyhow, Context};

use regex::Regex;
use text_file_sort::sort::Sort;

use crate::osm::apidb_dump::read::element_iterator::ElementIterator;
use crate::osm::apidb_dump::read::table_def::TableDef;
use crate::osm::apidb_dump::read::table_fields::TableFields;

/// Reader of apidb schema dump produced by pg_dump
pub struct Reader {
    tables: HashMap<String, TableDef>,
}

impl Reader {
    /// Create a new [Reader]
    ///
    /// * input_path - a path to directory that contains apidb schema dump produced by pg_dump with
    /// directory format. For example:
    /// ```bash
    ///  pg_dump --host localhost --port 5432 --username openstreetmap --no-password --file /result --format d -d openstreetmap --compress 0 --table public.nodes --table public.node_tags --table public.ways --table public.way_nodes --table public.way_tags --table public.relations --table public.relation_members --table public.relation_tags --table public.changesets --table public.users
    /// ```
    /// The input is sorted using primary keys for each table found in input_path/toc.dat which may
    /// take significant time depending on the size of the input
    /// * tmp_path - location used by the sorting algorithm for intermediate and final result. Should
    /// have space for at least 2.2 * input size
    pub fn new(input_path: PathBuf, tmp_path: PathBuf) -> Result<Reader, anyhow::Error> {
        let mut tables: HashMap<String, TableDef> = HashMap::new();

        let toc_path = input_path.join("toc.dat");
        let toc = fs::read(&toc_path)
            .with_context(|| anyhow!("path: {}", toc_path.to_string_lossy()))?;
        let raw_table_defs = Self::get_table_def_strings(&toc);
        // COPY public.node_tags (node_id, version, k, v) FROM stdin
        let re = Regex::new("^([^ ]+) \\((.+)\\)$").unwrap();
        for raw_table_def in raw_table_defs {
            let table_data_path = input_path.join(&raw_table_def.1);
            let captures = re.captures(&raw_table_def.0).unwrap();
            let name = captures.get(1).unwrap().as_str();
            if !TableFields::is_of_interest(name) {
                continue;
            }
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

        Self::sort_tables(&tables)?;

        Ok(
            Reader {
                tables,
            }
        )
    }

    fn sort_tables(tables: &HashMap<String, TableDef>) -> Result<(), anyhow::Error> {
        for (table_name, table_def) in tables {
            log::info!("Sort {} table data", table_name);
            std::fs::create_dir_all(table_def.tmp_path())?;
            let mut text_file = Sort::new(vec![table_def.path()], table_def.sorted_path());
            text_file.with_tmp_dir(table_def.tmp_path());
            text_file.with_intermediate_files(8192);
            text_file.with_tasks(num_cpus::get());
            text_file.with_fields(table_def.pkey().key());
            text_file.with_ignore_empty();
            text_file.with_ignore_lines(Regex::new("^\\\\\\.$")?);
            text_file.sort()?;
        }
        Ok(())
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

    /// Create iterator over the elements.
    ///
    /// The behaviour is similar to that of [osm::pbf::element_iterator::ElementIterator] with the
    /// distinction that [Element::Sentinel] is produced after completing each type, that is Node,
    /// Way, Relation.
    pub fn elements(&self) -> Result<ElementIterator, anyhow::Error> {
        ElementIterator::new(self.tables.clone())
    }
}



