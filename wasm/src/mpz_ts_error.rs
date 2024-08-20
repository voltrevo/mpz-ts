use thiserror::Error;

#[derive(Error, Debug)]
pub enum MpzTsError {
    #[error(transparent)]
    SerdeWasmError(#[from] serde_wasm_bindgen::Error),
}
