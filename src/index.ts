import { Circuit } from './frameworkTypes';
import { getWasmLib, initWasmLib } from './wasmLib';

export async function init(numThreads: number) {
  await initWasmLib(numThreads);
}

export function testEval(
  circuit: Circuit,
  inputs: Record<string, unknown>,
): Record<string, unknown> {
  return getWasmLib().test_eval(circuit, inputs);
}

export async function testAlice(
  send: (msg: Uint8Array) => void,
  recv: () => Uint8Array,
) {
  return await getWasmLib().test_alice(send, recv);
}

export async function testBob(
  send: (msg: Uint8Array) => void,
  recv: () => Uint8Array,
) {
  return await getWasmLib().test_bob(send, recv);
}
