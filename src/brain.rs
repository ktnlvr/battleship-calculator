use std::fmt::{Display, Write};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq)]
pub enum CellState {
    #[default]
    EMPTY,
    MISS,
    HIT,
    SUNK,
}

impl Display for CellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellState::EMPTY => Ok(()),
            CellState::MISS => f.write_char('o'),
            CellState::HIT => f.write_char('x'),
            CellState::SUNK => f.write_char('X'),
        }
    }
}

pub struct GridState {
    pub cells: Vec<Vec<CellState>>,
}

pub fn calculate_chances(
    grid: &Vec<Vec<CellState>>,
    grid_size: usize,
    ships: &Vec<usize>,
) -> Vec<Vec<usize>> {
    let n = grid_size;
    let mut chances = vec![vec![0usize; n]; n];
    let mut mask = vec![vec![true; n]; n];

    for i in 0..n {
        for j in 0..n {
            match grid[i][j] {
                CellState::EMPTY => {}
                CellState::MISS => mask[i][j] = false,
                CellState::HIT => {
                    for (x, y) in [(1, 1), (1, -1), (-1, -1), (-1, 1)] {
                        let (ix, x_overflow) = i.overflowing_add_signed(x);
                        let (jy, y_overflow) = j.overflowing_add_signed(y);

                        if x_overflow || y_overflow || ix == n || jy == n {
                            continue;
                        }

                        mask[ix][jy] = false;
                    }
                }
                CellState::SUNK => {
                    for (x, y) in [
                        (0, 0),
                        (0, 1),
                        (1, 0),
                        (0, -1),
                        (-1, 0),
                        (1, 1),
                        (1, -1),
                        (-1, -1),
                        (-1, 1),
                    ] {
                        let (ix, x_overflow) = i.overflowing_add_signed(x);
                        let (iy, y_overflow) = j.overflowing_add_signed(y);
                        if x_overflow || y_overflow || ix == n || iy == n {
                            continue;
                        }

                        mask[ix][iy] = false;
                    }
                }
            }
        }
    }

    for s in ships.iter().map(|x| *x) {
        if s > n {
            continue;
        }

        for i in 0..n {
            for j in 0..(n - s + 1) {
                if mask[i][j..(j + s)].iter().all(|b| *b) {
                    chances[i][j..(j + s)].iter_mut().for_each(|x| *x += 1);
                }

                if mask[j..(j + s)].iter().map(|row| row[i]).all(|x| x) {
                    chances[j..(j + s)].iter_mut().for_each(|row| row[i] += 1);
                }
            }
        }
    }

    chances
}
