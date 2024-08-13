use wasm_bindgen::{prelude::*, JsError};
use console_error_panic_hook::set_once as set_panic_hook;

#[wasm_bindgen]
pub fn init_ext() {
    set_panic_hook();
}

#[wasm_bindgen]
pub fn test(msg: &str) -> Result<JsValue, JsError> {
    Ok(msg.into())
}
