use wasm_bindgen::prelude::*;
use wasm_bindgen::convert::*;
use web_sys::*;

use std::sync::{Arc, Mutex};

mod cgol;
pub use cgol::*;

fn wrap<'a, T: FromWasmAbi, F: Fn(T) + 'static>(callback: F) -> ScopedClosure<'a, dyn Fn(T)> {
	Closure::wrap(Box::new(callback) as Box<dyn Fn(_)>)
}

fn draw(ctx: &CanvasRenderingContext2d, canvas: &Grid) {
	let width = ctx.canvas().unwrap().width();
	let height = ctx.canvas().unwrap().height();

	ctx.set_fill_style_str("#00ff00");
	ctx.fill_rect(0.0, 0.0, width as f64, height as f64);
	// ctx.clear_rect(0.0, 0.0, width as f64, height as f64);

	let cells = canvas.get_cells();
	let sz = 50.0;
	let m = 0.0;
	for (i, row) in cells.iter().enumerate() {
		for (j, c) in row.iter().enumerate() {
			let color = if c.is_alive() {
				"#f0f0f0"
			} else {
				"#0f0f0f"
			};

			if j == 15 {
				ctx.set_stroke_style_str("#00ffff");
			} else {
				ctx.set_stroke_style_str("#ff00ff");
			}

			let i = i as f64;
			let j = j as f64;

			ctx.set_fill_style_str(color);
			ctx.set_stroke_style_str("#808080");
			ctx.set_line_width(5.0);

			ctx.begin_path();
			ctx.rect(
				(sz + m) * j, (sz + m) * i,
				sz, sz,
			);
			ctx.stroke();
			ctx.fill();
		}
	}
}

#[wasm_bindgen]
pub fn run() {
	let w = window().unwrap();
	let document = w.document().unwrap();

	let canvas = document.query_selector("canvas").unwrap().unwrap()
		.dyn_into::<HtmlCanvasElement>().unwrap();
	let canvas = Arc::new(canvas);

	let ctx = canvas.get_context("2d").unwrap().unwrap()
		.dyn_into::<CanvasRenderingContext2d>().unwrap();
	let ctx = Arc::new(ctx);

	let mut grid = Grid::new(16, 16);
	grid.set_cell(1, 0, Cell::new(true));
	grid.set_cell(1, 1, Cell::new(true));
	grid.set_cell(1, 2, Cell::new(true));
	let grid = Arc::new(Mutex::new(grid));
	draw(ctx.clone().as_ref(), &grid.lock().unwrap());

	let onresize = {
		let ctx = ctx.clone();
		let w = w.window();
		let c = grid.clone();
		let canvas = canvas.clone();
		move |_: Event|{
			let width = w.inner_width().unwrap().as_f64().unwrap();
			let height = w.inner_height().unwrap().as_f64().unwrap();

			canvas.set_width(width as u32);
			canvas.set_height(height as u32);
			draw(ctx.as_ref(), &c.lock().unwrap());
		}
	};
	onresize(Event::new("resize").unwrap());

	let c = wrap(onresize);
	w.add_event_listener_with_callback("resize", &c.as_ref().unchecked_ref()).unwrap();
	c.forget();

	let update = {
		let ctx = ctx.clone();
		let grid = grid.clone();
		move |_: Event|{
			let grid: &mut Grid = &mut grid.lock().unwrap();
			grid.advance();
			draw(ctx.as_ref(), grid);
		}
	};
	let c = wrap(update);
	w.set_interval_with_callback_and_timeout_and_arguments_0(&c.as_ref().unchecked_ref(), 500).unwrap();
	c.forget();
}
