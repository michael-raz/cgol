mod cgol;
pub use cgol::*;

mod render;



#[wasm_bindgen::prelude::wasm_bindgen]
pub fn run() {
	render::run();
}
