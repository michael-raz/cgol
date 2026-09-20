use wasm_bindgen::prelude::*;
use wasm_bindgen::convert::*;
use web_sys::*;

use std::sync::{Arc, Mutex};

use crate::cgol::*;



macro_rules! consolelog {
	($($e:expr),+ $(,)?) => {
		console::log_1(&format!($($e),+).into())
	};
}



const CELL_SIZE: f64 = 50.0;

struct Viewer {
	grid: Grid,
	ctx: CanvasRenderingContext2d,

	viewport_dim: (f64, f64),
	camera_pos: (f64, f64),
	scale: f64,
}
impl Viewer {
	fn from_screen_space(&self, xy: (f64, f64)) -> (f64, f64) {
		(
			(xy.0 + self.camera_pos.0) / self.scale,
			(xy.1 + self.camera_pos.1) / self.scale,
		)
	}

	fn draw(&self) {
		let (width, height) = self.viewport_dim;

		const DEAD_COLOR:  &str = "#0f0f0f";
		const ALIVE_COLOR: &str = "#f0f0f0";

		self.ctx.set_fill_style_str(DEAD_COLOR);
		self.ctx.fill_rect(0.0, 0.0, width as f64, height as f64);
		// ctx.clear_rect(0.0, 0.0, width as f64, height as f64);

		let size = CELL_SIZE * self.scale;

		self.ctx.set_fill_style_str(ALIVE_COLOR);
		for pos in self.grid.get_alive() {
			self.ctx.fill_rect(
				size * (pos.x as f64) - self.camera_pos.0,
				size * (pos.y as f64) - self.camera_pos.1,
				size, size,
			);
		}
	}
}

fn wrap<'a, T: FromWasmAbi, F: FnMut(T) + 'static>(callback: F) -> ScopedClosure<'a, dyn FnMut(T)> {
	Closure::wrap(Box::new(callback) as Box<dyn FnMut(_)>)
}

fn add_event<T, E, F>(name: &'static str, target: &T, callback: F)
	where
		T: AsRef<EventTarget>,
		E: FromWasmAbi,
		F: FnMut(E) + 'static,
{
	let c = wrap(callback);
	target.as_ref().add_event_listener_with_callback(name, &c.as_ref().unchecked_ref()).unwrap();
	c.forget();
}

fn set_interval<E, F>(target: &Window, callback: F, rate: i32)
	where
		E: FromWasmAbi,
		F: FnMut(E) + 'static,
{
	let c = wrap(callback);
	target.set_interval_with_callback_and_timeout_and_arguments_0(
		&c.as_ref().unchecked_ref(), rate,
	).unwrap();
	c.forget();
}

pub fn run() {
	let w = window().unwrap();
	let document = w.document().unwrap();

	let canvas = document.query_selector("canvas").unwrap().unwrap()
		.dyn_into::<HtmlCanvasElement>().unwrap();
	let canvas = Arc::new(canvas);

	let ctx = canvas.get_context("2d").unwrap().unwrap()
		.dyn_into::<CanvasRenderingContext2d>().unwrap();

	// acorn
	let grid = Grid::from_bits(&[
		[0, 1, 0, 0, 0, 0 ,0],
		[0, 0, 0, 1, 0, 0 ,0],
		[1, 1, 0, 0, 1, 1 ,1],
	]);

	let viewer = Viewer{
		grid,
		ctx,
		viewport_dim: (0.0, 0.0),
		camera_pos: (0.0, 0.0),
		scale: 1.0,
	};
	let viewer = Arc::new(Mutex::new(viewer));
	viewer.lock().unwrap().draw();

	let onresize = {
		let w = w.window();
		let viewer = viewer.clone();
		let canvas = canvas.clone();
		move |_: Event|{
			let width = w.inner_width().unwrap().as_f64().unwrap();
			let height = w.inner_height().unwrap().as_f64().unwrap();

			let viewer: &mut Viewer = &mut viewer.lock().unwrap();
			viewer.viewport_dim = (width, height);

			canvas.set_width(width as u32);
			canvas.set_height(height as u32);
			viewer.draw();
		}
	};
	onresize(Event::new("resize").unwrap());
	add_event("resize", &w, onresize);

	add_event("mousemove", canvas.as_ref(), {
		let mut pinpoint: Option<(f64, f64)> = None;

		let viewer = viewer.clone();
		move |e: MouseEvent|{
			let viewer: &mut Viewer = &mut viewer.lock().unwrap();

			let pos = (
				e.x() as f64,
				e.y() as f64,
			);

			let lmb = (e.buttons() & 1) > 0;
			let shift = e.shift_key();

			if pinpoint.is_none() && lmb && !shift {
				let mut tmp = pos;
				tmp.0 += viewer.camera_pos.0;
				tmp.1 += viewer.camera_pos.1;

				pinpoint = Some(tmp);
			} else if !lmb {
				pinpoint = None;
			}

			if let Some(pinpoint) = pinpoint {
				viewer.camera_pos.0 = pinpoint.0 - pos.0;
				viewer.camera_pos.1 = pinpoint.1 - pos.1;

				viewer.draw();
			}
		}
	});

	add_event("mousedown", canvas.as_ref(), {
		let viewer = viewer.clone();
		move |e: MouseEvent|{
			let viewer: &mut Viewer = &mut viewer.lock().unwrap();

			let pos = (
				e.x() as f64,
				e.y() as f64,
			);

			let lmb = (e.buttons() & 1) > 0;
			let shift = e.shift_key();

			if lmb && shift {
				let pos = viewer.from_screen_space(pos);
				let pos = (
					(pos.0 / CELL_SIZE).floor() as i64,
					(pos.1 / CELL_SIZE).floor() as i64,
				).into();

				let alive = viewer.grid.get_cell(&pos);
				viewer.grid.set_cell(pos, !alive);

				viewer.draw();
			}
		}
	});

	// TODO: _also_ use "mousewheel" event to support Safari
	//       https://developer.mozilla.org/en-US/docs/Web/API/Element/mousewheel_event
	add_event("wheel", canvas.as_ref(), {
		let viewer = viewer.clone();
		move |e: WheelEvent|{
			let viewer = &mut viewer.lock().unwrap();

			let mpos = (e.x() as f64, e.y() as f64);

			let prev = viewer.scale;
			let delta = viewer.scale * -0.1 * (e.delta_y()).signum();
			viewer.scale += delta;

			viewer.camera_pos.0 = (viewer.scale / prev) * (mpos.0 + viewer.camera_pos.0) - mpos.0;
			viewer.camera_pos.1 = (viewer.scale / prev) * (mpos.1 + viewer.camera_pos.1) - mpos.1;

			viewer.draw();
		}
	});

	set_interval(&w, {
		let viewer = viewer.clone();
		move |_: Event|{
			let viewer: &mut Viewer = &mut viewer.lock().unwrap();
			viewer.grid.step(1);
			viewer.draw();
		}
	}, 500);
}
