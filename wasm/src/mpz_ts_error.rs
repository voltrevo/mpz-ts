use thiserror::Error;

#[derive(Error, Debug)]
pub enum MpzTsError {
    SerdeWasmError(#[from] serde_wasm_bindgen::Error),
}
