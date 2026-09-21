use std::sync::{Arc, Mutex};
use std::str::FromStr;

use crate::cgol::*;
use crate::wasm_helpers::*;
use crate::wasm_helpers::println;
use crate::dom::*;



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

	// NOTE: these are in viewer because
	//       panning and zooming both need to modify them
	cursor_pin: Option<(f64, f64)>,
	rsel: Option<((f64, f64), (f64, f64))>,
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
			rsel: None,
		}
	}

	fn from_screen_space(&self, xy: (f64, f64)) -> (f64, f64) {
		(
			(xy.0 + self.camera_pos.0) / self.scale / CELL_SIZE,
			(xy.1 + self.camera_pos.1) / self.scale / CELL_SIZE,
		)
	}

	fn to_screen_space(&self, xy: (f64, f64)) -> (f64, f64) {
		(
			xy.0 * self.scale * CELL_SIZE - self.camera_pos.0,
			xy.1 * self.scale * CELL_SIZE - self.camera_pos.1,
		)
	}

	fn draw(&self) {
		let (width, height) = self.viewport_dim;

		const DEAD_COLOR:  &str = "#0f0f0f";
		const ALIVE_COLOR: &str = "#f0f0f0";
		const RSEL_COLOR:  &str = "#ff0000";
		let size = CELL_SIZE * self.scale;

		self.ctx.set_fill_style_str(DEAD_COLOR);
		self.ctx.fill_rect(0.0, 0.0, width as f64, height as f64);

		// draw rectangle selection
		if let Some((start, end)) = self.rsel {
			let (start, end) = (
				(
					start.0.min(end.0),
					start.1.min(end.1),
				),
				(
					start.0.max(end.0),
					start.1.max(end.1),
				),
			);

			let start = (
				start.0.floor(),
				start.1.floor(),
			);
			let end = (
				end.0.ceil(),
				end.1.ceil(),
			);

			let start = self.to_screen_space(start);
			let end = self.to_screen_space(end);

			self.ctx.set_stroke_style_str(RSEL_COLOR);
			self.ctx.stroke_rect(
				start.0,
				start.1,
				end.0 - start.0,
				end.1 - start.1,
			);
		}

		// draw cells
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



pub fn run() {
	let w = window().unwrap();
	let document = w.document().unwrap();
	let body = WrappedHtml::own(document.body().unwrap().into());

	let div = WrappedHtml::new("div").unwrap();
	set_style!((&div){
		"display": "flex";
		"flex-direction": "column";
		"width": "100%";
		"height": "100%";
	});
	body.append_child(&div).unwrap();

	let canvas = WrappedHtml::new("canvas").unwrap();
	set_style!((&canvas){
		"width": "100%";
		"height": "100%";
	});
	div.append_child(&canvas).unwrap();

	let ctx = canvas.as_elm::<HtmlCanvasElement>().unwrap().get_context("2d").unwrap().unwrap()
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

	let canvas = Arc::new(canvas);
	init_canvas(canvas, viewer.clone());

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

	div.append_child(&create_controls(viewer, di)).unwrap();
}



fn init_canvas(canvas: Arc<WrappedHtml>, viewer: Arc<Mutex<Viewer>>) {
	// TODO: _also_ use "mousewheel" event to support Safari
	//       https://developer.mozilla.org/en-US/docs/Web/API/Element/mousewheel_event
	canvas.add_listener("wheel", {
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
	}).unwrap();


	canvas.add_listener("mousemove", {
		let viewer = viewer.clone();
		move |e: MouseEvent|{
			let viewer: &mut Viewer = &mut viewer.lock().unwrap();

			let mpos = (
				e.x() as f64,
				e.y() as f64,
			);

			let lmb = (e.buttons() & 1) > 0;
			let rmb = (e.buttons() & 2) > 0;
			let shift = e.shift_key();

			// camera pannning
			let pan = rmb && !shift;
			if viewer.cursor_pin.is_none() && pan {
				let mut tmp = mpos;
				tmp.0 += viewer.camera_pos.0;
				tmp.1 += viewer.camera_pos.1;

				viewer.cursor_pin = Some(tmp);
			} else if !pan {
				viewer.cursor_pin = None;
			}

			if let Some(pinpoint) = viewer.cursor_pin {
				viewer.camera_pos.0 = pinpoint.0 - mpos.0;
				viewer.camera_pos.1 = pinpoint.1 - mpos.1;
			}


			// rectangle selection
			let pos = viewer.from_screen_space(mpos);
			let rsel = lmb && shift;
			if viewer.rsel.is_none() && rsel {
				viewer.rsel = Some((pos, pos));
			} else if !rsel {
				viewer.rsel = None;
			} else if let Some(rsel) = viewer.rsel.as_mut() {
				rsel.1 = pos;
			}

			viewer.draw();
		}
	}).unwrap();


	// toggle cells
	canvas.add_listener("mousedown", {
		let viewer = viewer.clone();
		move |e: MouseEvent|{
			let viewer: &mut Viewer = &mut viewer.lock().unwrap();

			let pos = (
				e.offset_x() as f64,
				e.offset_y() as f64,
			);

			let lmb = (e.buttons() & 1) > 0;
			let shift = e.shift_key();

			if lmb && !shift {
				let pos = viewer.from_screen_space(pos);
				let pos = (
					pos.0.floor() as i64,
					pos.1.floor() as i64,
				).into();

				let alive = viewer.grid.get_cell(&pos);
				viewer.grid.set_cell(pos, !alive);

				viewer.draw();
			}
		}
	}).unwrap();


	canvas.add_listener("contextmenu", |e: Event| {
		e.prevent_default();
	}).unwrap();


	let onresize = {
		let viewer = viewer.clone();
		let canvas = canvas.clone();
		move |_: Event|{
			let canvas = canvas.as_elm::<HtmlCanvasElement>().unwrap();
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
	WrappedHtml::own(window().unwrap()).add_listener("resize", onresize).unwrap();
}



fn create_load_button(viewer: Arc<Mutex<Viewer>>) -> WrappedHtml {
	let button = WrappedHtml::new("button").unwrap();
	button.as_elm::<Node>().unwrap().set_text_content(Some("Load"));

	button.add_listener("click", {
		let viewer = viewer.clone();
		move |_: MouseEvent| {
			let w = window().unwrap();
			let nav = proto_get(&w, "navigator").unwrap();
			let clip = proto_get(nav.dyn_ref().unwrap(), "clipboard").unwrap();
			let read = proto_get(clip.dyn_ref().unwrap(), "readText").unwrap();
			let read = read.dyn_into::<Function>().unwrap();
			let text: Promise = read.call(&clip, ()).unwrap().dyn_into().unwrap();

			let viewer = viewer.clone();

			let c = wrap(move |text: JsValue| {
				use base64::prelude::*;
				use flate2::read::ZlibDecoder;
				use std::io::Read;

				let text = text.as_string().unwrap();
				let bytes = BASE64_STANDARD.decode(text).unwrap();
				let mut data = vec![];
				ZlibDecoder::new(&mut bytes.as_slice())
					.read_to_end(&mut data).unwrap();

				let mut viewer = viewer.lock().unwrap();
				viewer.grid = Grid::load(&mut data.as_slice()).unwrap();
			});
			let _ = text.then(&c);
			c.forget();
		}
	}).unwrap();

	button
}

fn create_save_button(viewer: Arc<Mutex<Viewer>>) -> WrappedHtml {
	let button = WrappedHtml::new("button").unwrap();
	button.as_elm::<Node>().unwrap().set_text_content(Some("Save"));

	button.add_listener("click", {
		let viewer = viewer.clone();
		move |_: MouseEvent| {
			let viewer = viewer.lock().unwrap();
			let mut raw = vec![];
			viewer.grid.save(&mut raw).unwrap();

			use flate2::{Compression, read::ZlibEncoder};
			use std::io::Read;

			let mut bytes = vec![];
			ZlibEncoder::new(&mut raw.as_slice(), Compression::best())
				.read_to_end(&mut bytes).unwrap();

			use base64::prelude::*;

			let out = BASE64_STANDARD.encode(bytes);

			let w = window().unwrap();
			let nav = proto_get(&w, "navigator").unwrap();
			let clip = proto_get(nav.dyn_ref().unwrap(), "clipboard").unwrap();
			let write = proto_get(clip.dyn_ref().unwrap(), "writeText").unwrap();
			let write = write.dyn_into::<Function>().unwrap();
			write.call(&clip, (&out.into(),)).unwrap();
		}
	}).unwrap();

	button
}

fn create_play_button(viewer: Arc<Mutex<Viewer>>) -> WrappedHtml {
	let button = WrappedHtml::new("button").unwrap();
	button.as_elm::<Node>().unwrap().set_text_content(Some("Play"));

	button.add_listener("click", {
		let viewer = viewer.clone();
		move |e: MouseEvent| {
			let mut viewer = viewer.lock().unwrap();
			viewer.paused = !viewer.paused;

			let node: Node = e.target().unwrap().dyn_into().unwrap();
			node.set_text_content(Some(if viewer.paused { "Play" } else { "Pause" }));
		}
	}).unwrap();

	button
}

const DEFAULT_SLIDE: i32 = 10;
fn from_slider(src: i32) -> Option<i32> {
	(src != 0).then(|| 50_000 / src / src)
}
fn create_slider<F: 'static + Fn()>(di: Arc<Mutex<DynamicInterval<F>>>, label: Arc<WrappedHtml>) -> WrappedHtml {
	let slider = WrappedHtml::new("input").unwrap();
	slider.set("type", &"range".into()).unwrap();
	slider.set("value", &DEFAULT_SLIDE.into()).unwrap();
	slider.add_listener("input", {
		let di = di.clone();
		let label = label.clone();
		move |e: Event| {
			let t = e.target().unwrap();

			let value = proto_get(t.as_ref(), "value").unwrap();
			let value = i32::from_str(&value.as_string().unwrap()).unwrap();
			let value = from_slider(value);

			label.as_elm::<Node>().unwrap().set_text_content(Some(&(if let Some(value) = value {
				format!("{:>5.2}", 1000.0 / value as f64)
			} else {
				"N/A".to_string()
			})));

			DynamicInterval::set_rate(di.clone(), value);
		}
	}).unwrap();

	slider
}

fn create_label() -> WrappedHtml {
	WrappedHtml::new("span").unwrap()
}

fn create_controls<F>(viewer: Arc<Mutex<Viewer>>, di: Arc<Mutex<DynamicInterval<F>>>) -> WrappedHtml
	where F: 'static + Fn()
{
	let controls = WrappedHtml::new("div").unwrap();

	let label = create_label();
	let label = Arc::new(label);

	controls.append_child(&create_slider(di, label.clone())).unwrap();
	controls.append_child(&label).unwrap();

	controls.append_child(&create_play_button(viewer.clone())).unwrap();
	controls.append_child(&create_save_button(viewer.clone())).unwrap();
	controls.append_child(&create_load_button(viewer.clone())).unwrap();

	controls
}
