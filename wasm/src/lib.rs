mod js_conn;
mod js_fn_executor;
mod mpz_circuit_from_bristol;
mod mpz_ts_error;
mod setup_garble;

use std::sync::Arc;

use bristol_circuit::{BristolCircuit, RawBristolCircuit};
use console_error_panic_hook::set_once as set_panic_hook;
use js_conn::JsConn;
use js_sys::Reflect;
use mpz_circuit_from_bristol::mpz_circuit_from_bristol;
use mpz_circuits::{circuits::AES128, types::Value};
use mpz_common::executor::STExecutor;
use mpz_garble::{value::ValueRef, Decode, DecodePrivate, Execute, Memory};
use mpz_ts_error::MpzTsError;
use serde_wasm_bindgen::from_value;
use serio::codec::{Bincode, Codec};
use setup_garble::Role;
use wasm_bindgen::prelude::*;

pub use wasm_bindgen_rayon::init_thread_pool;

#[wasm_bindgen]
pub fn init_ext() {
    set_panic_hook();
}

#[wasm_bindgen]
pub async fn run_semi_honest(
    circuit: JsValue,
    inputs: js_sys::Object,
    is_leader: bool,
    send: &js_sys::Function,
    recv: &js_sys::Function,
) -> Result<JsValue, JsError> {
    let bristol_circuit = BristolCircuit::from_raw(
        &from_value::<RawBristolCircuit>(circuit).map_err(Into::<MpzTsError>::into)?,
    )?;

    let ann_circuit = mpz_circuit_from_bristol(&bristol_circuit)?;

    let conn = JsConn::new(send, recv);
    let channel = Bincode.new_framed(conn);

    let role = if is_leader { Role::Alice } else { Role::Bob };

    // Create an executor and use it to instantiate a vm for garbled circuits.
    let executor = STExecutor::new(channel);

    let mut garble_vm = setup_garble::setup_garble(
        role,
        executor,
        // FIXME: Fix calculation. Seems to work with only 1 OT.
        32 * ann_circuit.inputs.len(),
    )
    .await
    .unwrap();

    let mut garble_inputs = Vec::<ValueRef>::new();

    for (input_name, input_width) in &ann_circuit.inputs {
        let input_value = js_sys::Reflect::get(&inputs, &JsValue::from(input_name))
            .map_err(|_| JsError::new("input lookup threw exception"))?;

        if input_value.is_undefined() {
            garble_inputs.push(garble_vm.new_blind_array_input::<bool>(&input_name, *input_width)?);
        } else {
            let input_value = as_uint(&input_value)?;

            let value_ref = garble_vm.new_private_array_input::<bool>(&input_name, *input_width)?;

            garble_vm.assign(
                &value_ref,
                usize_to_mpz_bit_array(input_value, *input_width),
            )?;

            garble_inputs.push(value_ref);
        }
    }

    let mut garble_outputs = Vec::<ValueRef>::new();

    for (output_name, output_width) in &ann_circuit.outputs {
        garble_outputs.push(garble_vm.new_array_output::<bool>(&output_name, *output_width)?);
    }

    // Execute the circuit.
    garble_vm
        .execute(
            Arc::new(ann_circuit.circuit),
            &garble_inputs,
            &garble_outputs,
        )
        .await
        .unwrap();

    // Decode outputs
    let outputs = garble_vm.decode(&garble_outputs).await.unwrap();

    let result = js_sys::Object::new();

    for ((name, _), value) in ann_circuit.outputs.iter().zip(outputs.iter()) {
        Reflect::set(
            &result,
            &JsValue::from(name),
            &JsValue::from(mpz_bit_array_to_usize(value)),
        )
        .map_err(|_| JsError::new("Failed to set output"))?;
    }

    Ok(result.into())
}

fn usize_to_mpz_bit_array(value: usize, width: usize) -> Value {
    Value::Array(
        (0..width)
            .rev()
            .map(|j| Value::Bit((value & (1 << j)) != 0))
            .collect(),
    )
}

fn mpz_bit_array_to_usize(value: &Value) -> usize {
    let bits = match value {
        Value::Array(bits) => bits,
        _ => panic!("Expected bit array"),
    };

    let mut res = 0;

    for (i, b) in bits.iter().rev().enumerate() {
        let b = match b {
            Value::Bit(false) => 0,
            Value::Bit(true) => 1,
            _ => panic!("Expected bit"),
        };

        res += b << i;
    }

    res
}

#[wasm_bindgen]
pub fn test_eval(circuit: JsValue, inputs: js_sys::Object) -> Result<JsValue, JsError> {
    let bristol_circuit = BristolCircuit::from_raw(
        &from_value::<RawBristolCircuit>(circuit).map_err(Into::<MpzTsError>::into)?,
    )?;

    let ann_circuit = mpz_circuit_from_bristol(&bristol_circuit)?;

    let mut mpz_inputs = Vec::<Value>::new();

    for (name, width) in &ann_circuit.inputs {
        let js_value = Reflect::get(&inputs, &JsValue::from(name))
            .map_err(|_| JsError::new(&format!("Failed to get input {}", name)))?;

        let value = as_uint(&js_value)?;
        mpz_inputs.push(usize_to_mpz_bit_array(value, *width));
    }

    let mpz_outputs = ann_circuit.circuit.evaluate(&mpz_inputs)?;

    let outputs = js_sys::Object::new();

    if ann_circuit.outputs.len() != mpz_outputs.len() {
        return Err(JsError::new("Output count mismatch"));
    }

    for ((name, _), value) in ann_circuit.outputs.iter().zip(mpz_outputs.iter()) {
        Reflect::set(
            &outputs,
            &JsValue::from(name),
            &JsValue::from(mpz_bit_array_to_usize(value)),
        )
        .map_err(|_| JsError::new("Failed to set output"))?;
    }

    Ok(outputs.into())
}

fn as_uint(js_value: &JsValue) -> Result<usize, JsError> {
    if js_value.is_undefined() {
        return Err(JsError::new("Undefined input"));
    }

    if js_value.is_null() {
        return Err(JsError::new("Null input"));
    }

    if let Some(bool) = js_value.as_bool() {
        return Ok(bool as usize);
    }

    if let Some(string) = js_value.as_string() {
        return string
            .parse()
            .map_err(|_| JsError::new("Failed to parse string"));
    }

    if let Some(number) = js_value.as_f64() {
        return Ok(number as usize);
    }

    Err(JsError::new("Invalid input"))
}

#[allow(dead_code)]
fn mpz_value_to_js_value(value: &Value) -> Result<JsValue, MpzTsError> {
    Ok(match value {
        Value::Bit(value) => JsValue::from(*value),
        Value::U8(value) => JsValue::from(*value),
        Value::U16(value) => JsValue::from(*value),
        Value::U32(value) => JsValue::from(*value),
        Value::U64(value) => JsValue::from(*value),
        Value::U128(value) => JsValue::from(*value),
        Value::Array(array) => {
            let js_array = js_sys::Array::new_with_length(array.len() as u32);

            for (i, value) in array.iter().enumerate() {
                js_array.set(i as u32, mpz_value_to_js_value(value)?);
            }

            js_array.into()
        }
        _ => return Err(MpzTsError::UnexpectedMpzValue),
    })
}

#[wasm_bindgen]
pub async fn test_alice(
    send: &js_sys::Function,
    recv: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    let conn = JsConn::new(send, recv);
    let channel = Bincode.new_framed(conn);

    // Create an executor and use it to instantiate a vm for garbled circuits.
    let executor = STExecutor::new(channel);
    let mut garble_vm = setup_garble::setup_garble(Role::Alice, executor, 256)
        .await
        .unwrap();

    // Define input and output types.
    let key = garble_vm.new_private_input::<[u8; 16]>("key").unwrap();
    let msg = garble_vm.new_blind_input::<[u8; 16]>("msg").unwrap();
    let ciphertext = garble_vm.new_output::<[u8; 16]>("ciphertext").unwrap();

    // Assign the key.
    garble_vm
        .assign(
            &key,
            [
                0x2b_u8, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09,
                0xcf, 0x4f, 0x3c,
            ],
        )
        .unwrap();

    // Load the AES circuit.
    let circuit = AES128.clone();

    // Execute the circuit.
    garble_vm
        .execute(circuit, &[key, msg], &[ciphertext.clone()])
        .await
        .unwrap();

    // Receive output information from Bob.
    let mut output = garble_vm.decode_private(&[ciphertext]).await.unwrap();

    // Print the encrypted text.
    let encrypted: [u8; 16] = output.pop().unwrap().try_into().unwrap();

    Ok(format!("Encrypted text is {:x?}", encrypted).into())
}

#[wasm_bindgen]
pub async fn test_bob(
    send: &js_sys::Function,
    recv: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    let conn = JsConn::new(send, recv);
    let channel = Bincode.new_framed(conn);

    // Create an executor and use it to instantiate a vm for garbled circuits.
    let executor = STExecutor::new(channel);
    let mut garble_vm = setup_garble::setup_garble(Role::Bob, executor, 256)
        .await
        .unwrap();

    // Define input and output types.
    let key = garble_vm.new_blind_input::<[u8; 16]>("key").unwrap();
    let msg = garble_vm.new_private_input::<[u8; 16]>("msg").unwrap();
    let ciphertext = garble_vm.new_output::<[u8; 16]>("ciphertext").unwrap();

    // Assign the message.
    garble_vm
        .assign(
            &msg,
            [
                0x6b_u8, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73,
                0x93, 0x17, 0x2a,
            ],
        )
        .unwrap();

    // Load the AES circuit.
    let circuit = AES128.clone();

    // Execute the circuit.
    garble_vm
        .execute(circuit, &[key, msg], &[ciphertext.clone()])
        .await
        .unwrap();

    // Send output information to Alice.
    garble_vm.decode_blind(&[ciphertext]).await.unwrap();

    Ok(JsValue::UNDEFINED)
}
