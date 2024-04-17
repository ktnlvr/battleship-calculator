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

    for i in 0..grid_size {
        for j in 0..grid_size {
            let i_f: f32 = i as f32;
            let j_f = j as f32;

            let ip = i_f / (grid_size as f32);
            let jp = j_f / (grid_size as f32);

            let cell = document.create_element_ns(Some("http://www.w3.org/2000/svg"), "rect")?;
            cell.set_attribute("x", format!("{ip}").as_str())?;
            cell.set_attribute("y", format!("{jp}").as_str())?;
            cell.set_attribute(
                "width",
                format!("{}", 1. / (grid_size as f32)).as_str(),
            )?;
            cell.set_attribute(
                "height",
                format!("{}", 1. / (grid_size as f32)).as_str(),
            )?;
            grid_div.append_child(&cell)?;
        }
    }

    Ok(())
}

fn main() {
    start().unwrap();
}
