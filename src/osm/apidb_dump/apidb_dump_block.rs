use crate::osm::model::element::Element;

pub struct ApidbDumpBlock {
    index: usize,
    elements: Vec<Element>,
}

impl ApidbDumpBlock {
    pub fn new(index: usize, elements: Vec<Element>) -> ApidbDumpBlock {
        ApidbDumpBlock {
            index,
            elements,
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn elements(&self) -> &Vec<Element> {
        &self.elements
    }

    pub fn take_elements(&mut self) -> Vec<Element> {
        std::mem::take(&mut self.elements)
    }
}