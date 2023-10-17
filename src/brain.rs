use std::{
    collections::HashSet,
    fmt::{Display, Write},
    iter::from_generator,
    ops::Generator,
};

use log::info;

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

pub fn extract_sunken_ships(grid: &Vec<Vec<CellState>>, grid_size: usize) -> Vec<usize> {
    let n = grid_size;

    let mut sunk = vec![];
    let mut near_queue = vec![(0, 0)];
    let mut visited = HashSet::new();

    while let Some((x, y)) = near_queue.pop() {
        if visited.contains(&(x, y)) {
            continue;
        }

        if grid[x][y] == CellState::SUNK {
            let mut this_connectedness = vec![];
            let mut sunken = vec![(x, y)];
            while let Some((i, j)) = sunken.pop() {
                if this_connectedness.contains(&(i, j)) {
                    continue;
                }

                sunken.extend(
                    from_generator(get_neumann_neighbors(n, i, j))
                        .filter(|&(x, y)| grid[x][y] == CellState::SUNK),
                );
                this_connectedness.push((i, j));
            }

            sunk.push(this_connectedness.len());
            visited.extend(this_connectedness);
        } else {
            visited.insert((x, y));
        }

        near_queue.extend(from_generator(get_neumann_neighbors(n, x, y)))
    }

    sunk
}

pub fn calculate_chances(
    grid: &Vec<Vec<CellState>>,
    grid_size: usize,
    ships: &Vec<usize>,
) -> Vec<Vec<usize>> {
    let n = grid_size;
    let mut chances = vec![vec![0usize; n]; n];
    let mut ship_may_be_here = vec![vec![true; n]; n];
    let ships = {
        let mut ships = ships.clone();
        for sunken_size in extract_sunken_ships(grid, grid_size) {
            if let Some(i) = ships.iter().position(|&ship_size| ship_size == sunken_size) {
                ships.remove(i);
            }
        }

        ships
    };

    for i in 0..n {
        for j in 0..n {
            match grid[i][j] {
                CellState::EMPTY => {}
                CellState::MISS => ship_may_be_here[i][j] = false,
                CellState::HIT => {
                    for (ix, iy) in from_generator(get_diagonal_neighbours(n, i, j)) {
                        ship_may_be_here[ix][iy] = false;
                    }
                }
                CellState::SUNK => {
                    ship_may_be_here[i][j] = false;
                    for (ix, iy) in from_generator(get_moore_neighbors(n, i, j)) {
                        ship_may_be_here[ix][iy] = false;
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
                if ship_may_be_here[i][j..(j + s)].iter().all(|b| *b) {
                    chances[i][j..(j + s)].iter_mut().for_each(|x| *x += 1);
                }

                if ship_may_be_here[j..(j + s)]
                    .iter()
                    .map(|row| row[i])
                    .all(|x| x)
                {
                    chances[j..(j + s)].iter_mut().for_each(|row| row[i] += 1);
                }
            }
        }
    }

    chances
}
