use wasm_bindgen::prelude::*;
use wasm_bindgen::convert::*;
use web_sys::*;

use std::sync::{Arc, Mutex};

mod cgol;
pub use cgol::*;

const CELL_SIZE: f64 = 50.0;

macro_rules! consolelog {
	($($e:expr),+) => {
		console::log_1(&format!($($e),+).into())
	};
}

struct Viewer {
	grid: Grid,
	viewport_dim: (f64, f64),
	camera_pos: (f64, f64),
	scale: f64,
}
impl Viewer {
	fn update_bounds(&mut self) {
		self.grid.set_bounds(
			(self.camera_pos.0.min(0.0) / CELL_SIZE / self.scale).abs().ceil() as u64,
			(self.camera_pos.1.min(0.0) / CELL_SIZE / self.scale).abs().ceil() as u64,
			((self.viewport_dim.0 + self.camera_pos.0.max(0.0)) / CELL_SIZE / self.scale).ceil() as u64,
			((self.viewport_dim.1 + self.camera_pos.1.max(0.0)) / CELL_SIZE / self.scale).ceil() as u64,
		);
	}

	fn from_screen_space(&self, xy: (f64, f64)) -> (f64, f64) {
		(
			(xy.0 + self.camera_pos.0) / self.scale,
			(xy.1 + self.camera_pos.1) / self.scale,
		)
	}
	fn to_screen_space(&self, xy: (f64, f64)) -> (f64, f64) {
		(
			xy.0 * self.scale - self.camera_pos.0,
			xy.1 * self.scale - self.camera_pos.1,
		)
	}
}

fn wrap<'a, T: FromWasmAbi, F: FnMut(T) + 'static>(callback: F) -> ScopedClosure<'a, dyn FnMut(T)> {
	Closure::wrap(Box::new(callback) as Box<dyn FnMut(_)>)
}

fn draw(ctx: &CanvasRenderingContext2d, viewer: &mut Viewer) {
	viewer.update_bounds();
	let (width, height) = viewer.viewport_dim;

	ctx.set_fill_style_str("#00ff00");
	ctx.fill_rect(0.0, 0.0, width as f64, height as f64);
	// ctx.clear_rect(0.0, 0.0, width as f64, height as f64);

	let grid = &viewer.grid;

	let mut start = viewer.from_screen_space((0.0, 0.0));
	let mut end = viewer.from_screen_space(viewer.viewport_dim);

	let cells = grid.get_cells();
	let origin = grid.get_origin();
	let size = CELL_SIZE * viewer.scale;

	start.0 = (start.0 / CELL_SIZE + origin.0 as f64).floor();
	start.1 = (start.1 / CELL_SIZE + origin.1 as f64).floor();
	end.0 = (end.0 / CELL_SIZE + origin.0 as f64).ceil();
	end.1 = (end.1 / CELL_SIZE + origin.1 as f64).ceil();

	for i in start.1 as usize..end.1 as usize {
		for j in start.0 as usize..end.0 as usize {
			let c = &cells[i][j];

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

			let i = i as f64 - grid.get_origin().1 as f64;
			let j = j as f64 - grid.get_origin().0 as f64;

			ctx.set_fill_style_str(color);
			ctx.set_stroke_style_str("#808080");
			ctx.set_line_width(5.0);

			ctx.begin_path();
			ctx.rect(
				size * j - viewer.camera_pos.0,
				size * i - viewer.camera_pos.1,
				size, size,
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
	let viewer = Viewer{
		grid,
		viewport_dim: (0.0, 0.0),
		camera_pos: (0.0, 0.0),
		scale: 1.0,
	};
	let viewer = Arc::new(Mutex::new(viewer));
	draw(ctx.clone().as_ref(), &mut viewer.lock().unwrap());

	let onresize = {
		let ctx = ctx.clone();
		let w = w.window();
		let viewer = viewer.clone();
		let canvas = canvas.clone();
		move |_: Event|{
			let width = w.inner_width().unwrap().as_f64().unwrap();
			let height = w.inner_height().unwrap().as_f64().unwrap();

			let viewer: &mut Viewer = &mut viewer.lock().unwrap();
			viewer.viewport_dim = (width, height);
			viewer.update_bounds();

			canvas.set_width(width as u32);
			canvas.set_height(height as u32);
			draw(ctx.as_ref(), viewer);
		}
	};
	onresize(Event::new("resize").unwrap());

	let c = wrap(onresize);
	w.add_event_listener_with_callback("resize", &c.as_ref().unchecked_ref()).unwrap();
	c.forget();

	let onmouse = {
		let mut pinpoint: Option<(f64, f64)> = None;

		let ctx = ctx.clone();
		let viewer = viewer.clone();
		move |e: MouseEvent|{
			let viewer: &mut Viewer = &mut viewer.lock().unwrap();

			let pos = (
				e.x() as f64,
				e.y() as f64,
			);

			if pinpoint.is_none() && (e.buttons() & 1) > 0 {
				let mut tmp = pos;
				tmp.0 += viewer.camera_pos.0;
				tmp.1 += viewer.camera_pos.1;

				pinpoint = Some(tmp);
			} else if (e.buttons() & 1) == 0 {
				pinpoint = None;
			}

			if let Some(pinpoint) = pinpoint {
				viewer.camera_pos.0 = pinpoint.0 - pos.0;
				viewer.camera_pos.1 = pinpoint.1 - pos.1;

				viewer.update_bounds();
				draw(ctx.as_ref(), viewer);
			}
		}
	};
	let c = wrap(onmouse);
	canvas.add_event_listener_with_callback("mousemove", &c.as_ref().unchecked_ref()).unwrap();
	c.forget();

	let onscroll = {
		let ctx = ctx.clone();
		let viewer = viewer.clone();
		move |e: WheelEvent|{
			// let viewer: &mut Viewer = {
			// 	viewer.try_lock
			// }
			let viewer = &mut viewer.lock().unwrap();

			let mpos = (e.x() as f64, e.y() as f64);

			let prev = viewer.scale;
			let delta = viewer.scale * -0.1 * (e.delta_y()).signum();
			viewer.scale += delta;

			viewer.camera_pos.0 = (viewer.scale / prev) * (mpos.0 + viewer.camera_pos.0) - mpos.0;
			viewer.camera_pos.1 = (viewer.scale / prev) * (mpos.1 + viewer.camera_pos.1) - mpos.1;

			viewer.update_bounds();
			draw(ctx.as_ref(), viewer);
		}
	};
	let c = wrap(onscroll);
	// TODO: _also_ use "mousewheel" event to support Safari
	//       https://developer.mozilla.org/en-US/docs/Web/API/Element/mousewheel_event
	canvas.add_event_listener_with_callback("wheel", &c.as_ref().unchecked_ref()).unwrap();
	c.forget();

	let update = {
		let ctx = ctx.clone();
		let viewer = viewer.clone();
		move |_: Event|{
			let viewer: &mut Viewer = &mut viewer.lock().unwrap();
			viewer.grid.advance();
			draw(ctx.as_ref(), viewer);
		}
	};
	let c = wrap(update);
	w.set_interval_with_callback_and_timeout_and_arguments_0(&c.as_ref().unchecked_ref(), 500).unwrap();
	c.forget();
}
