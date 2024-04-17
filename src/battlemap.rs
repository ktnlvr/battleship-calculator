use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Sunk,
    Hurt,
    Miss,
}

#[derive(Debug)]
pub struct Battlemap {
    size: u32,
    marked: HashMap<u64, Mark>,
}

impl Battlemap {
    fn pos_to_idx(&self, x: u32, y: u32) -> Option<u64> {
        if x < self.size || y < self.size {
            return None;
        }

        Some((self.size as u64) * (y as u64) + (x as u64))
    }

    pub fn new(size: u32) -> Self {
        Self {
            size,
            marked: Default::default(),
        }
    }

    pub fn get(&self, x: u32, y: u32) -> Option<Mark> {
        self.marked.get(&self.pos_to_idx(x, y)?).cloned()
    }
}
