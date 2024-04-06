use std::cell::RefCell;
use std::collections::HashMap;

use uuid::Uuid;

use crate::osm::model::element::Element;

thread_local! {
    static ACCUMULATORS: RefCell<HashMap<String, Vec<Element>>> = RefCell::new(HashMap::new());
}

/// Accumulate Elements to avoid calling [ParallelWriter] for each element
pub struct ThreadLocalAccumulator {
    id: String,
    capacity: usize,
}

impl ThreadLocalAccumulator {
    pub fn new(capacity: usize) -> ThreadLocalAccumulator {
        let id = Uuid::new_v4().to_string();
        ThreadLocalAccumulator {
            id,
            capacity,
        }
    }

    pub fn add(&self, element: Element) {
        ACCUMULATORS.with(|accumulators| {
            let mut accumulators = accumulators.borrow_mut();
            let accumulator = accumulators.get_mut(self.id.as_str());
            match accumulator {
                None => {
                    let mut acc = Vec::with_capacity(self.capacity);
                    acc.push(element);
                    accumulators.insert(self.id.clone(), acc);
                }
                Some(acc) => {
                    acc.push(element);
                }
            }
        });
    }

    pub fn elements(&self) -> Vec<Element> {
        ACCUMULATORS.with(|accumulators| {
            if !accumulators.borrow().contains_key(self.id.as_str()) {
                Vec::new()
            } else {
                let mut accumulators = accumulators.borrow_mut();
                let accumulator = accumulators.get_mut(self.id.as_str()).unwrap();
                std::mem::replace(accumulator, Vec::with_capacity(self.capacity))
            }
        })
    }

    #[allow(dead_code)]
    pub(crate) fn len(&self) -> usize {
        ACCUMULATORS.with(|accumulators| {
            if !accumulators.borrow().contains_key(self.id.as_str()) {
                0
            } else {
                let accumulators = accumulators.borrow_mut();
                let accumulator = accumulators.get(self.id.as_str()).unwrap();
                accumulator.len()
            }
        })
    }
}

