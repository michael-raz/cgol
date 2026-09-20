use wasm_bindgen::prelude::*;
use wasm_bindgen::convert::*;
use web_sys::*;
use js_sys::*;

use std::sync::{Arc, Mutex};
use std::str::FromStr;

use crate::cgol::*;
use crate::wasm_helpers::*;



macro_rules! println {
	() => {
		console::log_0();
	};
	($($arg:tt)*) => {
		console::log_1(&format!($($arg)*).into());
	};
}



const CELL_SIZE: f64 = 50.0;

struct DynamicInterval<F: Fn() + 'static> {
	rate: Option<u64>,
	callback: F,
	window: Window,
	prev: Option<i32>,
}
impl<F: Fn() + 'static> DynamicInterval<F> {
	fn new(window: Window, callback: F, rate: Option<u64>) -> Arc<Mutex<Self>> {
		let out = Arc::new(Mutex::new(Self {
			rate,
			callback,
			window,
			prev: None,
		}));

		Self::repeat(out.clone());

		return out;
	}

	fn repeat<T: Fn() + 'static>(me: Arc<Mutex<DynamicInterval<T>>>){
		let mut locked = me.lock().unwrap();
		if let Some(rate) = locked.rate {
			let cur = set_timeout(&locked.window, {
				let me = me.clone();
				move |_: Event| {
					(me.lock().unwrap().callback)();
					Self::repeat(me.clone());
				}
			}, rate as i32).unwrap();
			locked.prev.replace(cur);
		} else {
			locked.prev.take();
		}
	}

	fn set_rate(me: Arc<Mutex<Self>>, rate: Option<i32>) {
		let rate = rate.map(|n| n as u64);

		let mut locked = me.lock().unwrap();
		if locked.rate == rate {
			return;
		} else {
			locked.rate = rate;
		}

		if let Some(prev) = locked.prev {
			locked.window.clear_timeout_with_handle(prev);
		}

		drop(locked);

		Self::repeat(me);
	}
}

struct Viewer {
	grid: Grid,
	ctx: CanvasRenderingContext2d,
	paused: bool,

	viewport_dim: (u32, u32),
	camera_pos: (f64, f64),
	scale: f64,

	cursor_pin: Option<(f64, f64)>,
}
impl Viewer {
	fn new(grid: Grid, ctx: CanvasRenderingContext2d) -> Self {
		Self {
			grid,
			ctx,
			paused: true,
			viewport_dim: (0, 0),
			camera_pos: (0.0, 0.0),
			scale: 1.0,
			cursor_pin: None,
		}
	}

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

macro_rules! set_styles {
	($elm:expr, { $($name:literal: $value:expr);* $(;)? }) => {{
		let style = proto_get($elm, "style").and_then(|js| js.dyn_into::<Object>().ok()).unwrap();
		$( proto_set(&style, $name, &$value.into()).unwrap(); )*
	}};
}

pub fn run() {
	let w = window().unwrap();
	let document = w.document().unwrap();
	let body = document.body().unwrap();

	let div = document.create_element("div").unwrap();
	set_styles!(&div, {
		"display": "flex";
		"flex-direction": "column";
		"width": "100%";
		"height": "100%";
	});

	let canvas = Arc::new(document.create_element("canvas").unwrap()
		.dyn_into::<HtmlCanvasElement>().unwrap());
	div.append_child(&canvas).unwrap();

	set_styles!(&canvas, {
		"width": "100%";
		"height": "100%";
	});

	body.append_child(&div).unwrap();

	let ctx = canvas.get_context("2d").unwrap().unwrap()
		.dyn_into::<CanvasRenderingContext2d>().unwrap();

	// acorn
	let grid = Grid::from_bits(&[
		[0, 1, 0, 0, 0, 0 ,0],
		[0, 0, 0, 1, 0, 0 ,0],
		[1, 1, 0, 0, 1, 1 ,1],
	]);

	let viewer = Viewer::new(grid, ctx);
	let viewer = Arc::new(Mutex::new(viewer));
	viewer.lock().unwrap().draw();

	let onresize = {
		let viewer = viewer.clone();
		let canvas = canvas.clone();
		move |_: Event|{
			let viewer: &mut Viewer = &mut viewer.lock().unwrap();

			let width = canvas.client_width() as u32;
			let height = canvas.client_height() as u32;
			if viewer.viewport_dim.0 == width && viewer.viewport_dim.1 == height {
				return;
			}

			viewer.viewport_dim = (width, height);
			canvas.set_width(width);
			canvas.set_height(height);

			viewer.draw();
		}
	};
	onresize(Event::new("resize").unwrap());
	add_event("resize", &w, onresize);

	add_event("mousemove", canvas.as_ref(), {
		let viewer = viewer.clone();
		move |e: MouseEvent|{
			let viewer: &mut Viewer = &mut viewer.lock().unwrap();

			let mpos = (
				e.x() as f64,
				e.y() as f64,
			);

			let lmb = (e.buttons() & 1) > 0;
			let shift = e.shift_key();

			if viewer.cursor_pin.is_none() && lmb && !shift {
				let mut tmp = mpos;
				tmp.0 += viewer.camera_pos.0;
				tmp.1 += viewer.camera_pos.1;

				viewer.cursor_pin = Some(tmp);
			} else if !lmb {
				viewer.cursor_pin = None;
			}

			if let Some(pinpoint) = viewer.cursor_pin {
				viewer.camera_pos.0 = pinpoint.0 - mpos.0;
				viewer.camera_pos.1 = pinpoint.1 - mpos.1;

				viewer.draw();
			}
		}
	});

	add_event("mousedown", canvas.as_ref(), {
		let viewer = viewer.clone();
		move |e: MouseEvent|{
			let viewer: &mut Viewer = &mut viewer.lock().unwrap();

			let pos = (
				e.offset_x() as f64,
				e.offset_y() as f64,
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

			let mpos = (e.offset_x() as f64, e.offset_y() as f64);

			let prev = viewer.scale;
			let delta = viewer.scale * -0.1 * (e.delta_y()).signum();
			viewer.scale += delta;

			let before = viewer.camera_pos;
			viewer.camera_pos.0 = (viewer.scale / prev) * (mpos.0 + viewer.camera_pos.0) - mpos.0;
			viewer.camera_pos.1 = (viewer.scale / prev) * (mpos.1 + viewer.camera_pos.1) - mpos.1;

			if let Some(mut tmp) = viewer.cursor_pin {
				tmp.0 -= before.0 - viewer.camera_pos.0;
				tmp.1 -= before.1 - viewer.camera_pos.1;

				viewer.cursor_pin = Some(tmp);
			}

			viewer.draw();
		}
	});

	fn from_slider(src: i32) -> Option<i32> {
		(src != 0).then(|| 50_000 / src / src)
	}

	const DEFAULT_SLIDE: i32 = 10;

	let di = DynamicInterval::new(w, {
		let viewer = viewer.clone();
		move || {
			let viewer: &mut Viewer = &mut viewer.lock().unwrap();
			if !viewer.paused {
				viewer.grid.step(1);
				viewer.draw();
			}
		}
	}, None);
	DynamicInterval::set_rate(di.clone(), from_slider(DEFAULT_SLIDE));

	let controls = document.create_element("div").unwrap();

	let label = Arc::new(document.create_element("span").unwrap());

	let slider = document.create_element("input").unwrap();
	proto_set(&slider, "type", &"range".into()).unwrap();
	proto_set(&slider, "value", &DEFAULT_SLIDE.into()).unwrap();
	add_event("input", &slider, {
		let di = di.clone();
		let label = label.clone();
		move |e: Event| {
			let t = e.target().unwrap();

			let value = proto_get(t.as_ref(), "value").unwrap();
			let value = i32::from_str(&value.as_string().unwrap()).unwrap();
			let value = from_slider(value);

			label.set_text_content(Some(&(if let Some(value) = value {
				format!("{:>5.2}", 1000.0 / value as f64)
			} else {
				"N/A".to_string()
			})));

			DynamicInterval::set_rate(di.clone(), value);
		}
	});

	let play_toggle = {
		let button = document.create_element("button").unwrap();
		button.set_text_content(Some("Play"));

		add_event("click", &button, {
			let viewer = viewer.clone();
			move |e: MouseEvent| {
				let mut viewer = viewer.lock().unwrap();
				viewer.paused = !viewer.paused;

				let node: Node = e.target().unwrap().dyn_into().unwrap();
				node.set_text_content(Some(if viewer.paused { "Play" } else { "Pause" }));
			}
		});

		button
	};

	controls.append_child(&slider).unwrap();
	controls.append_child(&label).unwrap();
	controls.append_child(&play_toggle).unwrap();

	div.append_child(&controls).unwrap();
}
