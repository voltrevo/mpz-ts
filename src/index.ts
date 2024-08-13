import { getWasmLib, initWasmLib } from './wasmLib';

export async function init() {
  await initWasmLib();
}

export function test(
  msg: string,
): string {
  return getWasmLib().test(msg);
}
