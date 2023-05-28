use std::borrow::{Borrow, BorrowMut};
use log::Log;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use data_encoding::{HEXLOWER, HEXUPPER};
use num_format::Locale::el;
use crate::osm::model::element::Element;

thread_local! {
    static ACCUMULATORS: RefCell<HashMap<String, Vec<Element>>> = RefCell::new(HashMap::new());
}

pub struct ThreadLocalAccumulator {
    id: String,
    capacity: usize,
}

impl ThreadLocalAccumulator {
    pub fn new(capacity: usize) -> ThreadLocalAccumulator {
        let id = HEXUPPER.encode(&rand::random::<[u8; 16]>());
        ThreadLocalAccumulator {
            id,
            capacity,
        }
    }

    pub fn add(&self, element: Element) {
        ACCUMULATORS.with(|accumulators| {
            if !accumulators.borrow().contains_key(self.id.as_str()) {
                accumulators.borrow_mut().insert(self.id.clone(), Vec::with_capacity(self.capacity));
            }
            let mut accumulators = accumulators.borrow_mut();
            let accumulator = accumulators.get_mut(self.id.as_str()).unwrap();
            accumulator.push(element);
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

    pub fn len(&self) -> usize {
        ACCUMULATORS.with(|accumulators| {
            if !accumulators.borrow().contains_key(self.id.as_str()) {
                0
            } else {
                let mut accumulators = accumulators.borrow_mut();
                let accumulator = accumulators.get(self.id.as_str()).unwrap();
                accumulator.len()
            }
        })
    }
}

