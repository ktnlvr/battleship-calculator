use std::num::NonZeroUsize;

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;

use crate::get_document;

#[derive(Debug)]
pub struct Inputs {
    pub grid_size: usize,
    pub ships: Vec<usize>,
}

pub fn get_inputs() -> Inputs {
    let document = get_document();

    let grid_size = document
        .get_element_by_id("grid-size")
        .expect("No element with id `grid-size` found")
        .dyn_into::<HtmlInputElement>()
        .ok()
        .and_then(|inp| inp.value().parse::<usize>().ok())
        .and_then(NonZeroUsize::new)
        .map(NonZeroUsize::get)
        .unwrap_or(10);

    let ships = document
        .get_element_by_id("ships")
        .expect("No element with id `ships` found")
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
