use log::{debug, Level};
use wasm_bindgen::prelude::*;
use web_sys::*;

pub fn get_document() -> Document {
    web_sys::window()
        .expect("Couldn't get the window")
        .document()
        .expect("Couldn't get the document")
}

#[derive(Debug, Default)]
pub struct Inputs {
    pub grid_size: usize,
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

    Some(Inputs { grid_size })
}

#[wasm_bindgen(start)]
pub fn start() {
    main().unwrap();
}

pub fn regenerate_grid() -> Result<(), JsValue> {
    let document = get_document();
    let grid = document
        .get_element_by_id("grid")
        .expect("Couldn't get grid");

    let Some(inputs) = get_inputs() else {
        return Ok(());
    };

    let grid_header_row = document.create_element("tr")?;
    grid_header_row.append_child(document.create_element("th")?.as_ref())?;

    for i in 0..inputs.grid_size {
        let table_header = document.create_element("th")?;
        table_header.set_text_content(Some(&format!("{i}")));
        grid_header_row.append_child(&table_header)?;
    }

    grid.append_child(&grid_header_row)?;

    for i in 0..inputs.grid_size {
        let grid_row = document.create_element("tr")?;
        let column_marker = document.create_element("th")?;
        column_marker.set_text_content(Some(&format!("{i}")));
        grid_row.append_child(&column_marker)?;

        for j in 0..inputs.grid_size {
            let data_cell = document.create_element("td")?;
            data_cell.set_text_content(Some("X"));
            grid_row.append_child(&data_cell)?;
        }

        grid.append_child(&grid_row)?;
    }

    Ok(())
}

pub fn main() -> Result<(), JsValue> {
    console_log::init_with_level(Level::Debug).unwrap();
    console_error_panic_hook::set_once();
    debug!("Hello, world!");

    regenerate_grid()
}
