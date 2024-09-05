import * as mpzWasm from './src/wasmLib';
import { makeLocalCommsPair } from './tests/helpers/LocalComms';

async function main() {
  await mpzWasm.init(4);

  const [aliceComms, bobComms] = makeLocalCommsPair();

  const startTime = Date.now();

  const responses = await Promise.all([
    mpzWasm.testAlice(aliceComms.send, aliceComms.recv),
    mpzWasm.testBob(bobComms.send, bobComms.recv),
  ]);

  console.log(Date.now() - startTime, { responses });
}

main().catch(console.error);
