#![expect(special_module_name)]
mod main;

mod cgol;
pub use cgol::*;

mod render;
mod wasm_helpers;
mod dom;
