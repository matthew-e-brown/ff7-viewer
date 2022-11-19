pub mod ff7;
pub mod webgl;

use wasm_bindgen::prelude::*;
use web_sys::console;


// Use WeeAlloc instead of Rust's default allocator because it's roughly 10x smaller in size, albeit slower.
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    // Make Rust 'panics' log to JS console
    console_error_panic_hook::set_once();

    let (canvas, gl) = webgl::init_viewport("webgl-canvas");
    console::log_2(&canvas, &gl);

    Ok(())
}
