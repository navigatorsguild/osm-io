use ahash::AHashMap;

use crate::osmpbf::StringTable;

pub(crate) struct StringTableBuilder {
    string_table: Option<StringTable>,
    index: AHashMap<String, i32>,
}

impl StringTableBuilder {
    pub(crate) fn new() -> StringTableBuilder {
        let mut string_table: Option<StringTable> = Some(Default::default());
        string_table
            .as_mut()
            .unwrap()
            .s
            .push("".as_bytes().to_vec());
        StringTableBuilder {
            string_table,
            index: Default::default(),
        }
    }

    pub(crate) fn add(&mut self, s: &str) -> i32 {
        // Fast path: check without cloning
        if let Some(&idx) = self.index.get(s) {
            idx
        } else {
            // Slow path: only allocate on miss
            let string_table = self.string_table.as_mut().unwrap();
            string_table.s.push(s.as_bytes().to_vec());
            let idx = string_table.s.len() as i32 - 1;
            self.index.insert(s.to_string(), idx);
            idx
        }
    }

    pub(crate) fn build(&mut self) -> StringTable {
        let mut string_table: StringTable = Default::default();
        string_table.s.push("".as_bytes().to_vec());
        self.index.clear();
        self.string_table.replace(string_table).unwrap()
    }
}
