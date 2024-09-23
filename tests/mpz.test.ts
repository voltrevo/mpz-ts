import { expect } from 'chai';
import * as mpzWasm from '../src/wasmLib';
import { makeLocalCommsPair } from './helpers/LocalComms';
import blockTrim from './helpers/blockTrim';

describe('mpz', () => {
  // TODO: Tests need to run in bun not nodejs
  it.skip('test fn', async () => {
    await mpzWasm.init(4);

    const [aliceComms, bobComms] = makeLocalCommsPair();

    const responses = await Promise.all([
      mpzWasm.testAlice(aliceComms.send, aliceComms.recv),
      mpzWasm.testBob(bobComms.send, bobComms.recv),
    ]);

    console.log({ responses });

    // expect(msg).to.deep.eq(Uint8Array.from([0, 0, 0, 4, 2, 0, 0, 0]));
  });

  it('test eval', async () => {
    await mpzWasm.init(4);

    const output = mpzWasm.testEval(
      {
        bristol: blockTrim(`
          34 50
          2 8 8
          1 8

          2 1 7 15 49 XOR
          2 1 6 14 16 XOR
          2 1 7 15 17 AND
          2 1 16 17 48 XOR
          2 1 5 13 18 XOR
          2 1 6 14 19 AND
          2 1 17 16 20 AND
          2 1 19 20 21 OR
          2 1 18 21 47 XOR
          2 1 4 12 22 XOR
          2 1 5 13 23 AND
          2 1 21 18 24 AND
          2 1 23 24 25 OR
          2 1 22 25 46 XOR
          2 1 3 11 26 XOR
          2 1 4 12 27 AND
          2 1 25 22 28 AND
          2 1 27 28 29 OR
          2 1 26 29 45 XOR
          2 1 2 10 30 XOR
          2 1 3 11 31 AND
          2 1 29 26 32 AND
          2 1 31 32 33 OR
          2 1 30 33 44 XOR
          2 1 1 9 34 XOR
          2 1 2 10 35 AND
          2 1 33 30 36 AND
          2 1 35 36 37 OR
          2 1 34 37 43 XOR
          2 1 0 8 38 XOR
          2 1 1 9 39 AND
          2 1 37 34 40 AND
          2 1 39 40 41 OR
          2 1 38 41 42 XOR
        `),
        info: {
          "input_name_to_wire_index": {
            "b": 8,
            "a": 0
          },
          "constants": {},
          "output_name_to_wire_index": {
            "c": 42
          }
        },
      },
      {
        a: 150,
        b: 200,
      },
    );

    // 150 + 200 = 350
    // but in 8 bit, that wraps around to 94
    expect(output).to.deep.eq({ c: 94 });
  });
});
