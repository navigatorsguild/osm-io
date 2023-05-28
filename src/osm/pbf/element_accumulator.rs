use crate::osm::model::element::Element;

enum State {
    Nodes,
    Ways,
    Relations,
}

pub(crate) struct ElementAccumulator {
    block_size: usize,
    elements: Vec<Element>,
    state: State,
    index: usize,
}

impl ElementAccumulator {
    pub(crate) fn new() -> ElementAccumulator {
        let block_size = 8000 as usize;
        ElementAccumulator {
            block_size,
            elements: Vec::with_capacity(block_size),
            state: State::Nodes,
            index: 0,
        }
    }

    pub(crate) fn add(&mut self, element: Element) -> Option<Vec<Element>> {
        let mut result = None;
        match self.state {
            State::Nodes => {
                match &element {
                    Element::Node { .. } => {
                        self.elements.push(element);
                        if self.elements.len() == self.block_size {
                            result = Some(std::mem::replace(&mut self.elements, Vec::with_capacity(self.block_size)));
                            self.index += 1;
                        }
                    }
                    Element::Way { .. } => {
                        self.state = State::Ways;
                        result = Some(std::mem::replace(&mut self.elements, Vec::with_capacity(self.block_size)));
                        self.index += 1;
                        self.add(element);
                    }
                    Element::Relation { .. } => {
                        assert!(false, "expected Element::Node or Element::Way but got Element::Relation");
                    }
                    Element::Sentinel => {}
                }
            }
            State::Ways => {
                match &element {
                    Element::Node { .. } => {
                        assert!(false, "expected Element::Way or Element::Relation but got Element::Node");
                    }
                    Element::Way { .. } => {
                        self.elements.push(element);
                        if self.elements.len() == self.block_size {
                            result = Some(std::mem::replace(&mut self.elements, Vec::with_capacity(self.block_size)));
                            self.index += 1;
                        }
                    }
                    Element::Relation { .. } => {
                        self.state = State::Relations;
                        result = Some(std::mem::replace(&mut self.elements, Vec::with_capacity(self.block_size)));
                        self.index += 1;
                        self.add(element);
                    }
                    Element::Sentinel => {}
                }
            }
            State::Relations => {
                match &element {
                    Element::Node { .. } => {
                        assert!(false, "expected Element::Relation but got Element::Node");
                    }
                    Element::Way { .. } => {
                        assert!(false, "expected Element::Relation but got Element::Way");
                    }
                    Element::Relation { .. } => {
                        self.elements.push(element);
                        if self.elements.len() == self.block_size {
                            result = Some(std::mem::replace(&mut self.elements, Vec::with_capacity(self.block_size)));
                            self.index += 1;
                        }
                    }
                    Element::Sentinel => {}
                }
            }
        }
        result
    }

    pub(crate) fn elements(&mut self) -> Vec<Element> {
        self.index += 1;
        std::mem::take(&mut self.elements)
    }

    pub(crate) fn index(&self) -> usize {
        self.index
    }

    pub(crate) fn len(&self) -> usize {
        self.elements.len()
    }
}