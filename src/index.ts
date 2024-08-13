import { getWasmLib, initWasmLib } from './wasmLib';

export async function init() {
  await initWasmLib();
}

export async function test(
  send: (msg: Uint8Array) => void,
  recv: (maxSize: number) => Uint8Array,
) {
  await getWasmLib().test(send, recv);
}
