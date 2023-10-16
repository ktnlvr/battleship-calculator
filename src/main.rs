use std::{num::NonZeroUsize, sync::Mutex};

use lazy_static::lazy_static;
use log::Level;
use url::Url;
use wasm_bindgen::prelude::*;
use web_sys::*;

mod brain;
use brain::*;

lazy_static! {
    pub static ref GRID: Mutex<GridState> = Mutex::new(GridState { cells: vec![] });
}

pub fn get_document() -> Document {
    window()
        .expect("Couldn't get the window")
        .document()
        .expect("Couldn't get the document")
}

#[derive(Debug)]
pub struct Inputs {
    pub grid_size: usize,
    pub ships: Vec<usize>,
}

pub fn get_inputs() -> Inputs {
    let document = get_document();

    let grid_size = document
        .get_element_by_id("grid-size")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .ok()
        .and_then(|inp| inp.value().parse::<usize>().ok())
        .and_then(NonZeroUsize::new)
        .map(NonZeroUsize::get)
        .unwrap_or(10);

    let ships = document
        .get_element_by_id("ships")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .ok()
        .and_then(|inp| {
            inp.value()
                .split_ascii_whitespace()
                .map(|s| s.parse::<usize>())
                .collect::<Result<Vec<usize>, _>>()
                .ok()
        })
        .unwrap_or_default();

    let ships = if ships.is_empty() {
        vec![4, 3, 3, 2, 2, 2]
    } else {
        ships
    };

    Inputs { grid_size, ships }
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

    let cell_click_closure = Closure::wrap(Box::new(move || {
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

        refresh();
    }) as Box<dyn FnMut()>);

    cell.add_event_listener_with_callback("click", cell_click_closure.as_ref().unchecked_ref())?;
    cell_click_closure.forget();

    Ok(())
}

pub fn refresh() {
    let inputs = get_inputs();

    let chances = calculate_chances(&GRID.lock().unwrap().cells, inputs.grid_size, &inputs.ships);

    display_chances(chances);
}

pub fn display_chances(chances: Vec<Vec<usize>>) {
    let document = get_document();
    let inputs = get_inputs();

    let max_cell_chance = chances
        .iter()
        .map(|row| row.iter().max())
        .max()
        .flatten()
        .copied()
        .unwrap_or_default();

    for (i, row) in chances.iter().enumerate() {
        for (j, chance) in row.iter().enumerate().take(inputs.grid_size) {
            let cell = document
                .get_element_by_id(&format!("{i}x{j}"))
                .expect("Could not find a grip element at expected index!");

            if GRID.lock().unwrap().cells[i][j] != CellState::EMPTY {
                continue;
            }

            if *chance == max_cell_chance {
                cell.set_class_name("top-guess");
            } else {
                cell.set_class_name("");
            }

            cell.set_text_content(Some(&format!("{}", chance)))
        }
    }
}

pub fn regenerate_grid() -> Result<(), JsValue> {
    let document = get_document();
    let grid = document
        .get_element_by_id("grid")
        .expect("Couldn't get grid");
    let inputs = get_inputs();

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
pub fn start() -> Result<(), JsValue> {
    console_log::init_with_level(Level::Debug).unwrap();
    console_error_panic_hook::set_once();

    let document = get_document();
    let url = Url::parse(&document.url()?).unwrap();

    let grid_size_query_param =
        url.query_pairs()
            .find_map(|(param_name, value)| if param_name == "n" { Some(value) } else { None });
    let ships_query_param = url.query_pairs().find_map(|(param_name, value)| {
        if param_name == "ships" {
            Some(value)
        } else {
            None
        }
    });

    let react_to_input_change_closure = Closure::wrap(Box::new(move || {
        let document = get_document();

        let grid = document
            .get_element_by_id("grid")
            .expect("Couldn't get grid");

        let grid_parent = grid.parent_element().unwrap();
        grid.remove();

        let new_grid = document.create_element("table").unwrap();
        new_grid.set_id("grid");
        grid_parent.append_child(&new_grid).unwrap();

        let inputs = get_inputs();
        let ships_str = inputs.ships.iter().map(|x| format!("{x}")).collect::<Vec<_>>().join(" ");
        let url = window().unwrap().origin();

        window().unwrap().history().unwrap().replace_state_with_url(&JsValue::UNDEFINED, "!!!", Some(&format!("{}?n={}&ships={}", url, inputs.grid_size, ships_str))).unwrap();
        regenerate_grid().unwrap();

        refresh();
    }) as Box<dyn FnMut()>);

    // Ships input
    {
        let ships_input = document.get_element_by_id("ships").unwrap();

        ships_input.add_event_listener_with_callback(
            "change",
            react_to_input_change_closure.as_ref().unchecked_ref(),
        )?;

        if let Some(ships_query_param) = ships_query_param {
            ships_input
                .dyn_into::<HtmlInputElement>()?
                .set_value(&ships_query_param);
        }
    }

    // Grid size input
    {
        let grid_size_input = document.get_element_by_id("grid-size").unwrap();

        grid_size_input.add_event_listener_with_callback(
            "change",
            react_to_input_change_closure.as_ref().unchecked_ref(),
        )?;

        if let Some(grid_size_query_param) = grid_size_query_param {
            grid_size_input
                .dyn_into::<HtmlInputElement>()?
                .set_value(&grid_size_query_param);
        }
    }

    react_to_input_change_closure.forget();

    regenerate_grid()?;
    refresh();

    Ok(())
}

pub fn main() {
    start().unwrap();
}
