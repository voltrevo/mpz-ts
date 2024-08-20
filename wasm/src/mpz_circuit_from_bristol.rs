use bristol_circuit::{BristolCircuit, RawBristolCircuit};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::JsValue;

use crate::mpz_ts_error::MpzTsError;
use mpz_circuits::Circuit as MpzCircuit;

pub fn mpz_circuit_from_bristol(circuit: JsValue) -> Result<MpzCircuit, MpzTsError> {
    let raw_circuit = from_value::<RawBristolCircuit>(circuit)?;
    let circuit = BristolCircuit::from_raw(&raw_circuit)?;

    todo!()
}
