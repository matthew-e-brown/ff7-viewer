use wasm_bindgen::prelude::*;
use web_sys::console;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Make Rust 'panics' log to JS console
    console_error_panic_hook::set_once();
    console::log_1(&"Hello, World!".into());
    Ok(())
}
