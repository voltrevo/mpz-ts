use bristol_circuit::Gate;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MpzTsError {
    #[error(transparent)]
    SerdeWasmError(#[from] serde_wasm_bindgen::Error),
    #[error(transparent)]
    BristolCircuitError(#[from] bristol_circuit::BristolCircuitError),
    #[error("Unsupported op: {op}")]
    UnsupportedOp { op: String },
    #[error("Invalid gate: {gate}: {message}")]
    InvalidGate { gate: Gate, message: String },
    #[error("Output wire not found: {wire_index}")]
    OutputWireNotFound { wire_index: usize },
    #[error(transparent)]
    MpzCircuitBuilderError(#[from] mpz_circuits::BuilderError),
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("Unexpected value")]
    UnexpectedMpzValue,
}
