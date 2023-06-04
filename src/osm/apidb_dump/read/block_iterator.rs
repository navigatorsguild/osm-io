use std::collections::HashMap;
use std::ops::AddAssign;
use crate::osm::apidb_dump::apidb_dump_block::ApidbDumpBlock;
use crate::osm::apidb_dump::read::element_iterator::ElementIterator;
use crate::osm::apidb_dump::read::table_def::TableDef;
use crate::osm::model::element::Element;

pub struct _BlockIterator {
    element_iterator: ElementIterator,
    current_index: usize,
}

impl _BlockIterator {
    pub fn new(tables: HashMap<String, TableDef>) -> Result<_BlockIterator, anyhow::Error> {
        let element_iterator = ElementIterator::new(tables)?;
        Ok(
            _BlockIterator { element_iterator, current_index: 0 }
        )
    }
}

impl Iterator for _BlockIterator {
    type Item = ApidbDumpBlock;

    fn next(&mut self) -> Option<Self::Item> {
        let mut elements: Vec<Element> = Vec::with_capacity(8000);
        let mut result = None;
        while let Some(element) = self.element_iterator.next() {
            if let Element::Sentinel = &element {
                result = Some(
                    ApidbDumpBlock::new(
                        self.current_index,
                        elements,
                    )
                );
                self.current_index.add_assign(1);
                break;
            } else {
                elements.push(element);
                if elements.len() == 8000 {
                    result = Some(
                        ApidbDumpBlock::new(
                            self.current_index,
                            elements,
                        )
                    );
                    self.current_index.add_assign(1);
                    break;
                }
            }
        }
        result
    }
}