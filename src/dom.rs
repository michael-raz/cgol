use crate::wasm_helpers::*;



macro_rules! set_style {
	(($elm:expr) { $($name:literal: $value:expr);* $(;)? }) => {{
		$elm.get("style")
			.and_then(|js| js.dyn_into::<Object>().ok())
			$( .and_then(|obj| {
				proto_set(&obj, $name, &$value.into())?;
				Some(obj)
			}) )*;
	}};
}
pub(crate) use set_style;



pub struct WrappedHtml {
	raw: Element,
}
impl WrappedHtml {
	pub fn new(tag: &'static str) -> Result<Self, JsValue> {
		let wnd = window().unwrap();
		let doc = wnd.document().unwrap();

		let elm = doc.create_element(tag)?;

		return Ok(Self{raw: elm});
	}

	pub fn get(&self, key: &str) -> Option<JsValue> {
		proto_get(&self.raw, key)
	}
	pub fn set(&self, key: &str, value: &JsValue) -> Option<()> {
		proto_set(&self.raw, key, value.as_ref())
	}

	pub fn append_child(&self, child: &Self) -> Result<Node, JsValue> {
		self.raw.append_child(child.raw.as_ref())
	}

	pub fn add_listener<T, F>(&self, event_name: &'static str, callback: F) -> Result<(), JsValue>
		where
			// this _technically_ doesn't have to be static, but we need to keep track of the ref to the closure.
			// this would be a lot of book-keeping and we moslty use them statically anyways.
			F: 'static + FnMut(T),
			T: FromWasmAbi,
	{
		let callback = ScopedClosure::wrap(Box::new(callback));
		let jsfunc = callback.as_ref().dyn_ref().ok_or(JsValue::NULL)?;

		let target: &EventTarget = self.raw.as_ref();
		target.add_event_listener_with_callback(event_name, &jsfunc)?;

		// is static _and_ it's in js's control, so best to have rust forget about it
		callback.forget();

		return Ok(());
	}

	pub fn as_elm<T: JsCast>(&self) -> Option<&'_ T> {
		self.raw.dyn_ref()
	}

	pub fn own(elm: Element) -> Self {
		Self{raw: elm}
	}

	pub fn into_inner(self) -> Element {
		self.raw
	}
}
