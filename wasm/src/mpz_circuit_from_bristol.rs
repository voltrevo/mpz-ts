use std::collections::HashMap;

use bristol_circuit::{BristolCircuit, Gate};

use crate::mpz_ts_error::MpzTsError;
use mpz_circuits::{types::Bit, Circuit as MpzCircuit, CircuitBuilder, Tracer};

pub struct AnnotatedMpzCircuit {
    pub circuit: MpzCircuit,
    pub inputs: Vec<(String, usize)>,
    pub outputs: Vec<(String, usize)>,
}

pub fn mpz_circuit_from_bristol(
    circuit: &BristolCircuit,
) -> Result<AnnotatedMpzCircuit, MpzTsError> {
    let builder = CircuitBuilder::new();

    let (input_widths, output_widths) = match &circuit.io_widths {
        Some(io_widths) => io_widths,
        None => return Err(MpzTsError::ArithmeticCircuitNotSupported),
    };

    if !circuit.info.constants.is_empty() {
        // Only arithmetic circuits have constants
        return Err(MpzTsError::ArithmeticCircuitNotSupported);
    }

    let mut ordered_input_names = circuit
        .info
        .input_name_to_wire_index
        .iter()
        .map(|(name, i)| (name.clone(), i.clone()))
        .collect::<Vec<_>>();

    ordered_input_names.sort_by(|(_, i), (_, j)| i.cmp(j));

    if ordered_input_names.len() != input_widths.len() {
        return Err(MpzTsError::IoMismatch);
    }

    let mut ordered_output_names = circuit
        .info
        .output_name_to_wire_index
        .iter()
        .map(|(name, i)| (name.clone(), i.clone()))
        .collect::<Vec<_>>();

    ordered_output_names.sort_by(|(_, i), (_, j)| i.cmp(j));

    if ordered_output_names.len() != output_widths.len() {
        return Err(MpzTsError::IoMismatch);
    }

    let mut nodes = HashMap::<usize, Tracer<Bit>>::new();
    let mut inputs = Vec::<(String, usize)>::new();
    let mut outputs = Vec::<(String, usize)>::new();

    for (i, (name, wire_index)) in ordered_input_names.iter().enumerate() {
        let width = input_widths[i];

        for (j, tracer_bit) in builder.add_vec_input::<bool>(width).iter().enumerate() {
            nodes.insert(*wire_index + j, tracer_bit.clone());
        }

        inputs.push((name.clone(), width));
    }

    for gate in &circuit.gates {
        match gate.op.as_str() {
            "AND" => {
                let (x, y, z_index) = get_binary_io(gate, &nodes)?;
                let z = x & y;
                nodes.insert(z_index, z);
            }
            "XOR" => {
                let (x, y, z_index) = get_binary_io(gate, &nodes)?;
                let z = x ^ y;
                nodes.insert(z_index, z);
            }
            "OR" => {
                let (x, y, z_index) = get_binary_io(gate, &nodes)?;
                let z = x | y;
                nodes.insert(z_index, z);
            }
            "NOT" => {
                let (x, out_index) = get_unary_io(gate, &nodes)?;
                let out = !x;
                nodes.insert(out_index, out);
            }
            "COPY" => {
                let (x, out_index) = get_unary_io(gate, &nodes)?;
                nodes.insert(out_index, x);
            }
            _ => {
                return Err(MpzTsError::UnsupportedOp {
                    op: gate.op.clone(),
                })
            }
        }
    }

    for (i, (name, wire_index)) in ordered_output_names.iter().enumerate() {
        let width = output_widths[i];

        let mut bit_nodes = Vec::<Tracer<'_, Bit>>::new();

        for j in 0..width {
            let wire_index_j = *wire_index + j;

            let node = nodes
                .get(&wire_index_j)
                .ok_or_else(|| MpzTsError::OutputWireNotFound {
                    wire_index: wire_index_j,
                })?;

            bit_nodes.push(node.clone())
        }

        builder.add_output(bit_nodes);

        outputs.push((name.clone(), width));
    }

    Ok(AnnotatedMpzCircuit {
        circuit: builder.build()?,
        inputs,
        outputs,
    })
}

fn get_binary_io<'a>(
    gate: &Gate,
    nodes: &HashMap<usize, Tracer<'a, Bit>>,
) -> Result<(Tracer<'a, Bit>, Tracer<'a, Bit>, usize), MpzTsError> {
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

fn get_unary_io<'a>(
    gate: &Gate,
    nodes: &HashMap<usize, Tracer<'a, Bit>>,
) -> Result<(Tracer<'a, Bit>, usize), MpzTsError> {
    if gate.inputs.len() != 1 {
        return Err(MpzTsError::InvalidGate {
            gate: gate.clone(),
            message: "Unary gate must have exactly 1 input".into(),
        });
    }

    if gate.outputs.len() != 1 {
        return Err(MpzTsError::InvalidGate {
            gate: gate.clone(),
            message: "Unary gate must have exactly 1 output".into(),
        });
    }

    let x = nodes
        .get(&gate.inputs[0])
        .ok_or_else(|| MpzTsError::InvalidGate {
            gate: gate.clone(),
            message: "Input wire not found".into(),
        })?;

    Ok((*x, gate.outputs[0]))
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
