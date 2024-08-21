use std::{collections::HashMap, ops::BitOr};

use bristol_circuit::{BristolCircuit, Gate};

use crate::mpz_ts_error::MpzTsError;
use mpz_circuits::{
    ops::{WrappingAdd, WrappingSub},
    types::U32,
    Circuit as MpzCircuit, CircuitBuilder, Tracer,
};

pub struct AnnotatedMpzCircuit {
    pub circuit: MpzCircuit,
    pub input_names: Vec<String>,
    pub output_names: Vec<String>,
}

pub fn mpz_circuit_from_bristol(
    circuit: &BristolCircuit,
) -> Result<AnnotatedMpzCircuit, MpzTsError> {
    let builder = CircuitBuilder::new();

    let mut nodes = HashMap::<usize, Tracer<U32>>::new();
    let mut input_names = Vec::<String>::new();
    let mut output_names = Vec::<String>::new();

    for (name, wire_index) in &circuit.info.input_name_to_wire_index {
        nodes.insert(*wire_index, builder.add_input::<u32>());
        input_names.push(name.clone());
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

    for (name, wire_index) in &circuit.info.output_name_to_wire_index {
        let node = nodes
            .get(wire_index)
            .ok_or_else(|| MpzTsError::OutputWireNotFound {
                wire_index: *wire_index,
            })?;

        builder.add_output(node.clone());

        output_names.push(name.clone());
    }

    Ok(AnnotatedMpzCircuit {
        circuit: builder.build()?,
        input_names,
        output_names,
    })
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

#[cfg(test)]
mod tests {
    use bristol_circuit::{CircuitInfo, RawBristolCircuit};
    use mpz_circuits::types::Value;

    use super::*;

    fn clean(src: &str) -> String {
        src.trim_start()
            .trim_end_matches(char::is_whitespace)
            .lines()
            .map(str::trim)
            .collect::<Vec<&str>>()
            .join("\n")
            + "\n"
    }

    fn make_a_plus_b() -> BristolCircuit {
        BristolCircuit::from_raw(&RawBristolCircuit {
            bristol: clean(
                "
                1 3
                2 1 1
                1 1

                2 1 0 1 2 AAdd
            ",
            ),
            info: CircuitInfo {
                input_name_to_wire_index: [("a".to_string(), 0), ("b".to_string(), 1)]
                    .iter()
                    .cloned()
                    .collect(),
                constants: Default::default(),
                output_name_to_wire_index: [("c".to_string(), 2)].iter().cloned().collect(),
            },
        })
        .unwrap()
    }

    #[test]
    fn test_mpz_circuit_from_bristol() {
        let circuit = make_a_plus_b();
        let mpz_circuit = mpz_circuit_from_bristol(&circuit).unwrap().circuit;

        let output = mpz_circuit
            .evaluate(&[Value::U32(3), Value::U32(5)])
            .unwrap();

        assert_eq!(output, vec![Value::U32(8)]);
    }
}
