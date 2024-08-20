use serde_wasm_bindgen::from_value;
use wasm_bindgen::JsValue;

use crate::{circuit::Circuit, mpz_ts_error::MpzTsError};
use mpz_circuits::Circuit as MpzCircuit;

pub fn mpz_circuit_from_bristol(circuit: JsValue) -> Result<MpzCircuit, MpzTsError> {
    let circuit = from_value::<Circuit>(circuit)?;

    todo!()
}
