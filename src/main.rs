use battlemap::{Battlemap, CellData};
use log::Level;
use wasm_bindgen::prelude::*;
use web_sys::*;

mod battlemap;

fn get_document() -> Document {
    window().unwrap().document().unwrap()
}

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    console_log::init_with_level(Level::Debug).expect("Failed to initialize the console logger");
    console_error_panic_hook::set_once();

    let document = get_document();
    let grid_div = document.get_element_by_id("grid").unwrap();
    let children = grid_div.children();
    for i in 0..children.length() {
        let child = children.item(i).unwrap();
        grid_div.remove_child(&child).unwrap();
    }

    let grid_size = 10u32;

    let map = Battlemap::new(grid_size, [4, 3, 3, 2, 2, 2, 1, 1, 1, 1]);

    for i in 0..grid_size {
        for j in 0..grid_size {
            let CellData { mark, ship_overlap, hit_chance } = map.get(i, j).unwrap();

            let i_f: f32 = i as f32;
            let j_f = j as f32;

            let ip = i_f / (grid_size as f32);
            let jp = j_f / (grid_size as f32);

            let colour = colorous::VIRIDIS.eval_continuous(hit_chance);

            let cell = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "rect")?;
            cell.set_attribute("x", format!("{ip}").as_str())?;
            cell.set_attribute("y", format!("{jp}").as_str())?;
            cell.set_attribute("width", format!("{}", 1. / (grid_size as f32)).as_str())?;
            cell.set_attribute("height", format!("{}", 1. / (grid_size as f32)).as_str())?;
            cell.set_attribute("fill", format!("#{:X}", colour).as_str())?;
            cell.set_attribute("x-mark", format!("{:?}", mark).as_str())?;
            cell.set_attribute("x-overlap", format!("{}", ship_overlap).as_str())?;
            grid_div.append_child(&cell)?;
        }
    }

    Ok(())
}

fn main() {
    start().unwrap();
}
