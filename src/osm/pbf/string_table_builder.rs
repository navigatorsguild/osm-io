use std::collections::HashMap;
use crate::osmpbf::StringTable;

pub(crate) struct StringTableBuilder {
    string_table: Option<StringTable>,
    index: HashMap<String, i32>,
}

impl StringTableBuilder {
    pub(crate) fn new() -> StringTableBuilder {
        let mut string_table: Option<StringTable> = Some(Default::default());
        string_table.as_mut().unwrap().s.push("".as_bytes().to_vec());
        StringTableBuilder {
            string_table,
            index: Default::default(),
        }
    }

    pub(crate) fn add(&mut self, s: &String) -> i32 {
        let string_index: i32;
        if self.index.contains_key(s.as_str()) {
            string_index = *self.index.get(s).unwrap()
        } else {
            let key = s.clone();
            self.string_table.as_mut().unwrap().s.push(key.as_bytes().to_vec());
            string_index = self.string_table.as_ref().unwrap().s.len() as i32 - 1;
            self.index.insert(key, string_index);
        }
        string_index
    }

    pub(crate) fn build(&mut self) -> StringTable {
        let mut string_table: StringTable = Default::default();
        string_table.s.push("".as_bytes().to_vec());
        self.index.clear();
        self.string_table.replace(string_table).unwrap()
    }
}