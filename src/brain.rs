use std::{
    fmt::{Display, Write},
    ops::Generator,
};

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

const DIAGONAL_NEIGHBORHOOD: [(isize, isize); 4] = [(1, 1), (1, -1), (-1, -1), (-1, 1)];

fn neighbors(
    neighborhood: &'static [(isize, isize)],
    grid_size: usize,
    i: usize,
    j: usize,
) -> impl Generator<Yield = (usize, usize), Return = ()> {
    let n = grid_size;
    move || {
        for (x, y) in neighborhood {
            let (ix, x_overflow) = i.overflowing_add_signed(*x);
            let (jy, y_overflow) = j.overflowing_add_signed(*y);

            if x_overflow || y_overflow || ix == n || jy == n {
                continue;
            }

            yield (ix, jy);
        }
    }
}

pub fn get_diagonal_neighbours(
    n: usize,
    i: usize,
    j: usize,
) -> impl Generator<Yield = (usize, usize), Return = ()> {
    neighbors(&DIAGONAL_NEIGHBORHOOD, n, i, j)
}

const MOORE_NEIGHBORHOOD: [(isize, isize); 8] = [
    (0, 1),
    (1, 0),
    (0, -1),
    (-1, 0),
    (1, 1),
    (1, -1),
    (-1, -1),
    (-1, 1),
];

pub fn get_moore_neighbors(
    n: usize,
    i: usize,
    j: usize,
) -> impl Generator<Yield = (usize, usize), Return = ()> {
    neighbors(&MOORE_NEIGHBORHOOD, n, i, j)
}

const NEUMANN_NEIGHBORHOOD: [(isize, isize); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

pub fn get_neumann_neighbors(
    n: usize,
    i: usize,
    j: usize,
) -> impl Generator<Yield = (usize, usize), Return = ()> {
    neighbors(&NEUMANN_NEIGHBORHOOD, n, i, j)
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
                    for (ix, iy) in std::iter::from_generator(get_diagonal_neighbours(n, i, j)) {
                        mask[ix][iy] = false;
                    }
                }
                CellState::SUNK => {
                    mask[i][j] = false;
                    for (ix, iy) in std::iter::from_generator(get_moore_neighbors(n, i, j)) {
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
