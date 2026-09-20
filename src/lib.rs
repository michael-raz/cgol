mod cgol;
pub use cgol::*;

mod render;
mod wasm_helpers;



#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run() {
	render::run();
}
