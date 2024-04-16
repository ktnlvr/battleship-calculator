use log::Level;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn start() -> Result<(), JsValue> {
    console_log::init_with_level(Level::Debug).expect("Failed to initialize the console logger");
    console_error_panic_hook::set_once();

    Ok(())
}

fn main() {
    start().unwrap();
}
