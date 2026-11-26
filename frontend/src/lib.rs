//! ModelRadar Leptos WASM Frontend Entrypoint
//!
//! Production-Ready WebAssembly Client

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}
