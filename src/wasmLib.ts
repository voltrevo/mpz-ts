import * as bindgen from '../srcWasm/mpz_ts_wasm';

function base64ToUint8Array(base64: string) {
  var binaryString = atob(base64);
  var len = binaryString.length;
  var bytes = new Uint8Array(len);
  for (var i = 0; i < len; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes;
}

let promise: Promise<typeof bindgen> | undefined = undefined;
let lib: typeof bindgen | undefined = undefined;

export function initWasmLib() {
  promise ??= (async () => {
    const { default: wasmBase64 } = await import(
      '../srcWasm/mpz_ts_wasm_base64',
    );

    bindgen.initSync(base64ToUint8Array(wasmBase64));
    bindgen.init_ext();
    bindgen.initThreadPool((globalThis as any).navigator?.hardwareConcurrency ?? 4);
    lib = bindgen;

    return bindgen;
  })();

  return promise;
}

export function getWasmLib() {
  if (lib === undefined) {
    throw new Error('lib not initialized, call mpz.init() first');
  }

  return lib;
}
