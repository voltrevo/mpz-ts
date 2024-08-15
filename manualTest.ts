import * as mpz from './src';
import { makeLocalCommsPair } from './tests/helpers/LocalComms';

async function main() {
  await mpz.init(4);

  const [aliceComms, bobComms] = makeLocalCommsPair();

  const responses = await Promise.all([
    mpz.testAlice(aliceComms.send, aliceComms.recv),
    mpz.testBob(bobComms.send, bobComms.recv),
  ]);

  console.log({ responses });
}

main().catch(console.error);
