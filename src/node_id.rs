#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct NodeId {
    index: usize,
    generation: i32,
}

impl NodeId {
    pub(crate) fn new(index: usize) -> Self {
        Self {
            index,
            generation: 0,
        }
    }

    pub fn index(self) -> usize {
        self.index
    }

    pub(crate) fn next_gen(self) -> Self {
        Self {
            index: self.index,
            generation: self.generation + 1,
        }
    }

    pub fn generation(self) -> i32 {
        self.generation
    }
}
