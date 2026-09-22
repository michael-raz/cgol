#[cfg(target_family="wasm")]
use crate::wasm_helpers::{println, eprintln};

#[cfg(target_family="wasm")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn main() {
	println!("start");
	std::panic::set_hook(Box::new(|pinfo| {
		let mut out = String::new();
		if let Some(loc) = pinfo.location() {
			out += format!(
				"panicked at {}:{}:{}:\n",
				loc.file(), loc.line(), loc.column(),
			).as_str();
		} else {
			out += "panicked at an unknown location\n";
		}

		if let Some(msg) = pinfo.payload_as_str() {
			out += msg;
			out += "\n";
		}

		eprintln!("{}", out.trim_end());
	}));

	crate::render::run();
}

#[cfg(not(target_family="wasm"))]
pub fn main() {
	todo!();
}
