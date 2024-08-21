use std::{collections::HashMap, ops::BitOr};

use bristol_circuit::{BristolCircuit, Gate, RawBristolCircuit};
use serde_wasm_bindgen::from_value;
use wasm_bindgen::JsValue;

use crate::mpz_ts_error::MpzTsError;
use mpz_circuits::{
    ops::{WrappingAdd, WrappingSub},
    types::U32,
    Circuit as MpzCircuit, CircuitBuilder, Tracer,
};

pub fn mpz_circuit_from_bristol(circuit: JsValue) -> Result<MpzCircuit, MpzTsError> {
    let raw_circuit = from_value::<RawBristolCircuit>(circuit)?;
    let circuit = BristolCircuit::from_raw(&raw_circuit)?;

    let builder = CircuitBuilder::new();

    let mut nodes = HashMap::<usize, Tracer<U32>>::new();

    for (_, wire_index) in &circuit.info.input_name_to_wire_index {
        nodes.insert(*wire_index, builder.add_input::<u32>());
    }

    for (_, info) in &circuit.info.constants {
        nodes.insert(
            info.wire_index,
            builder.get_constant::<u32>(info.value.parse()?),
        );
    }

    for gate in &circuit.gates {
        match gate.op.as_str() {
            "AAdd" => {
                let (x, y, z_index) = get_binary_io(gate, &nodes)?;
                let z = x.wrapping_add(y);
                nodes.insert(z_index, z);
            }
            "ASub" => {
                let (x, y, z_index) = get_binary_io(gate, &nodes)?;
                let z = x.wrapping_sub(y);
                nodes.insert(z_index, z);
            }
            "ABitOr" => {
                let (x, y, z_index) = get_binary_io(gate, &nodes)?;
                let z = x.bitor(y);
                nodes.insert(z_index, z);
            }
            _ => {
                return Err(MpzTsError::UnsupportedOp {
                    op: gate.op.clone(),
                })
            }
        }
    }

    for (_, wire_index) in &circuit.info.output_name_to_wire_index {
        let node = nodes
            .get(wire_index)
            .ok_or_else(|| MpzTsError::OutputWireNotFound {
                wire_index: *wire_index,
            })?;

        builder.add_output(node.clone());
    }

    Ok(builder.build()?)
}

fn get_binary_io<'a>(
    gate: &Gate,
    nodes: &HashMap<usize, Tracer<'a, U32>>,
) -> Result<(Tracer<'a, U32>, Tracer<'a, U32>, usize), MpzTsError> {
    if gate.inputs.len() != 2 {
        return Err(MpzTsError::InvalidGate {
            gate: gate.clone(),
            message: "Binary gate must have exactly 2 inputs".into(),
        });
    }

    if gate.outputs.len() != 1 {
        return Err(MpzTsError::InvalidGate {
            gate: gate.clone(),
            message: "Binary gate must have exactly 1 output".into(),
        });
    }

    let x = nodes
        .get(&gate.inputs[0])
        .ok_or_else(|| MpzTsError::InvalidGate {
            gate: gate.clone(),
            message: "Input wire not found".into(),
        })?;

    let y = nodes
        .get(&gate.inputs[1])
        .ok_or_else(|| MpzTsError::InvalidGate {
            gate: gate.clone(),
            message: "Output wire not found".into(),
        })?;

    Ok((*x, *y, gate.outputs[0]))
}
