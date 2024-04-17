use std::{cmp::min, collections::HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Sunk,
    Hurt,
    Miss,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CellData {
    pub mark: Option<Mark>,
    pub ship_overlap: u32,
    pub hit_chance: f64,
}

#[derive(Debug)]
pub struct Battlemap {
    size: u32,
    ships: Box<[u32]>,

    marked: HashMap<u64, Mark>,
    ship_overlap_map: Box<[u32]>,
}

impl Battlemap {
    fn pos_to_idx(&self, x: u32, y: u32) -> Option<u64> {
        if x >= self.size || y >= self.size {
            return None;
        }

        Some((self.size as u64) * (y as u64) + (x as u64))
    }

    fn index_to_pos(&self, idx: u64) -> Option<(u32, u32)> {
        if idx % (self.size * self.size) as u64 != 0 {
            return None;
        }

        Some((
            (idx % self.size as u64) as u32,
            (idx / self.size as u64) as u32,
        ))
    }

    fn max_overlap(&self) -> u32 {
        self.ship_overlap_map.iter().copied().max().unwrap_or(0)
    }

    fn min_overlap(&self) -> u32 {
        self.ship_overlap_map.iter().copied().min().unwrap_or(0)
    }

    pub fn new(size: u32, ships: impl IntoIterator<Item = u32>) -> Self {
        let mut ship_overlap_map = vec![0; (size * size) as usize].into_boxed_slice();
        let ships = ships.into_iter().collect::<Vec<_>>().into_boxed_slice();

        for i in 0..size {
            for j in 0..size {
                let idx = (j * size + i) as usize;

                for s in ships.iter().copied() {
                    ship_overlap_map[idx] += min(i + 1, size - i).min(s);
                    ship_overlap_map[idx] += min(j + 1, size - j).min(s);
                }
            }
        }

        Self {
            size,
            marked: Default::default(),

            ships,
            ship_overlap_map,
        }
    }

    pub fn get(&self, x: u32, y: u32) -> Option<CellData> {
        let idx = self.pos_to_idx(x, y)?;
        let mark = self.marked.get(&idx).cloned();
        let ship_overlap = self.ship_overlap_map[idx as usize];
        let max_overlap = self.max_overlap();
        let min_overlap = self.min_overlap();

        Some(CellData {
            mark,
            ship_overlap,
            hit_chance: ((ship_overlap - min_overlap) as f64) / ((max_overlap - min_overlap) as f64),
        })
    }
}
