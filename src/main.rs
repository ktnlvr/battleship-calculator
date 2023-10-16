use std::{
    fmt::{Display, Write},
    sync::Mutex,
};

use lazy_static::lazy_static;
use log::{debug, info, Level};
use wasm_bindgen::prelude::*;
use web_sys::*;

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
            CellState::EMPTY => f.write_char(' '),
            CellState::MISS => f.write_char('o'),
            CellState::HIT => f.write_char('x'),
            CellState::SUNK => f.write_char('X'),
        }
    }
}

pub struct GridState {
    pub cells: Vec<Vec<CellState>>,
}

lazy_static! {
    pub static ref GRID: Mutex<GridState> = Mutex::new(GridState { cells: vec![] });
}

pub fn get_document() -> Document {
    web_sys::window()
        .expect("Couldn't get the window")
        .document()
        .expect("Couldn't get the document")
}

#[derive(Debug, Default)]
pub struct Inputs {
    pub grid_size: usize,
    pub ships: Vec<usize>,
}

pub fn get_inputs() -> Option<Inputs> {
    let document = get_document();

    let grid_size = document
        .get_element_by_id("grid-size")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .ok()
        .map(|inp| inp.value().parse::<usize>().ok())
        .flatten()?;

    let ships = document
        .get_element_by_id("ships")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .ok()
        .map(|inp| {
            inp.value()
                .split_ascii_whitespace()
                .map(|s| s.parse::<usize>())
                .collect::<Result<Vec<usize>, _>>()
                .ok()
        })
        .flatten()?;

    Some(Inputs { grid_size, ships })
}

fn name_from_number(n: usize) -> String {
    let i = n % 26;
    let ch = char::from_u32(65 + i as u32).unwrap();
    if n / 26 > 0 {
        format!("{}{ch}", name_from_number(n / 26 - 1))
    } else {
        ch.into()
    }
}

pub fn configure_cell(cell: &Element, x: usize, y: usize) -> Result<(), JsValue> {
    cell.set_text_content(None);
    cell.set_class_name("");

    let id = format!("{x}x{y}");
    cell.set_id(&id);

    let f = Closure::wrap(Box::new(move || {
        let doc = get_document();
        let cell = doc.get_element_by_id(&id).unwrap();

        {
            let cell_value = &mut GRID.lock().unwrap().cells[x][y];

            *cell_value = match *cell_value {
                CellState::EMPTY => CellState::MISS,
                CellState::MISS => CellState::HIT,
                CellState::HIT => CellState::SUNK,
                CellState::SUNK => CellState::EMPTY,
            };

            match *cell_value {
                CellState::EMPTY => cell.set_class_name(""),
                CellState::MISS => cell.set_class_name("miss"),
                CellState::HIT => cell.set_class_name("hit"),
                CellState::SUNK => cell.set_class_name("sunk"),
            }

            cell.set_text_content(Some(&format!("{cell_value}")));
        }

        if let Some(chances) = recalculate_chances() {
            display_chances(chances);
        }
    }) as Box<dyn FnMut()>);

    cell.add_event_listener_with_callback("click", f.as_ref().unchecked_ref())?;
    f.forget();

    Ok(())
}

pub fn recalculate_chances() -> Option<Vec<Vec<usize>>> {
    let inputs = get_inputs()?;
    let n = inputs.grid_size;

    let mut chances = vec![vec![0usize; inputs.grid_size]; inputs.grid_size];
    let mut mask = vec![vec![true; inputs.grid_size]; inputs.grid_size];

    let grid = &GRID.lock().unwrap().cells;

    for i in 0..n {
        for j in 0..n {
            match grid[i][j] {
                CellState::EMPTY => {},
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

    for s in inputs.ships {
        if s > n {
            continue;
        }

        for i in 0..n {
            for j in 0..(n - s + 1) {
                if mask[i][j..(j + s)].iter().all(|b| *b) {
                    (&mut chances[i][j..(j + s)])
                        .iter_mut()
                        .for_each(|x| *x += 1);
                }

                if mask[j..(j + s)].iter().map(|row| row[i]).all(|x| x) {
                    chances[j..(j + s)].iter_mut().for_each(|row| row[i] += 1);
                }
            }
        }
    }

    Some(chances)
}

pub fn display_chances(chances: Vec<Vec<usize>>) {
    let document = get_document();
    let Some(inputs) = get_inputs() else {
        return;
    };

    for i in 0..inputs.grid_size {
        for j in 0..inputs.grid_size {
            if GRID.lock().unwrap().cells[i][j] != CellState::EMPTY {
                continue;
            }

            let cell = document
                .get_element_by_id(&format!("{i}x{j}"))
                .expect("Could not find a grip element at expected index!");

            cell.set_text_content(Some(&format!("{}", chances[i][j])))
        }
    }
}

pub fn regenerate_grid() -> Result<(), JsValue> {
    let document = get_document();
    let grid = document
        .get_element_by_id("grid")
        .expect("Couldn't get grid");

    let Some(inputs) = get_inputs() else {
        return Ok(());
    };

    GRID.lock().unwrap().cells = vec![vec![CellState::EMPTY; inputs.grid_size]; inputs.grid_size];

    let grid_header_row = document.create_element("tr")?;
    grid_header_row.append_child(document.create_element("th")?.as_ref())?;

    for i in 0..inputs.grid_size {
        let table_header = document.create_element("th")?;
        table_header.set_text_content(Some(&name_from_number(i)));
        grid_header_row.append_child(&table_header)?;
    }

    grid.append_child(&grid_header_row)?;

    for i in 0..inputs.grid_size {
        let grid_row = document.create_element("tr")?;
        let column_marker = document.create_element("th")?;
        column_marker.set_text_content(Some(&format!("{}", i + 1)));
        grid_row.append_child(&column_marker)?;

        for j in 0..inputs.grid_size {
            let cell = document.create_element("td")?;
            configure_cell(&cell, i, j)?;
            grid_row.append_child(&cell)?;
        }

        grid.append_child(&grid_row)?;
    }

    Ok(())
}

#[wasm_bindgen(start)]
pub fn start() {
    main().unwrap();
}

pub fn main() -> Result<(), JsValue> {
    console_log::init_with_level(Level::Debug).unwrap();
    console_error_panic_hook::set_once();

    regenerate_grid()?;
    if let Some(chances) = recalculate_chances() {
        display_chances(chances);
    }

    Ok(())
}
