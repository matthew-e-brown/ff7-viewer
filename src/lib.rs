pub mod ff7;
pub mod webgl;

use std::io::Cursor;

use wasm_bindgen::prelude::*;
use web_sys::console;


#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Make Rust 'panics' log to JS console
    console_error_panic_hook::set_once();

    // Setup logger
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));

    let (canvas, gl) = webgl::init_viewport("webgl-canvas");
    console::log_2(&canvas, &gl);

    let temp_char_lgp = include_bytes!("../char.lgp");
    let mut cursor = Cursor::new(temp_char_lgp);

    let _result = ff7::LgpArchive::from_reader(&mut cursor);
    Ok(())
}
