use anyhow::{Result, anyhow};
use wasm_bindgen::{JsCast, JsValue};

pub trait JsResultExt<T> {
    /// Converts a Result<T, JsValue> into an anyhow::Result<T>
    fn map_js_err(self) -> Result<T>;
}

impl<T> JsResultExt<T> for Result<T, JsValue> {
    fn map_js_err(self) -> Result<T> {
        self.map_err(|err| {
            if let Some(js_err) = err.dyn_ref::<js_sys::Error>() {
                anyhow!(String::from(js_err.to_string()))
            } else {
                anyhow!("{err:?}")
            }
        })
    }
}
