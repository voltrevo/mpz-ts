import { getWasmLib, initWasmLib } from './wasmLib';

export async function init(numThreads: number) {
  await initWasmLib(numThreads);
}

export async function testAlice(
  send: (msg: Uint8Array) => void,
  recv: (maxSize: number) => Uint8Array,
) {
  return await getWasmLib().test_alice(send, recv);
}

export async function testBob(
  send: (msg: Uint8Array) => void,
  recv: (maxSize: number) => Uint8Array,
) {
  return await getWasmLib().test_bob(send, recv);
}
