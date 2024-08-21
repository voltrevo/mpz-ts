import { expect } from 'chai';
import * as mpz from '../src';
import { makeLocalCommsPair } from './helpers/LocalComms';
import blockTrim from './helpers/blockTrim';

describe('mpz', () => {
  // TODO: Tests need to run in bun not nodejs
  it.skip('test fn', async () => {
    await mpz.init(4);

    const [aliceComms, bobComms] = makeLocalCommsPair();

    const responses = await Promise.all([
      mpz.testAlice(aliceComms.send, aliceComms.recv),
      mpz.testBob(bobComms.send, bobComms.recv),
    ]);

    console.log({ responses });

    // expect(msg).to.deep.eq(Uint8Array.from([0, 0, 0, 4, 2, 0, 0, 0]));
  });

  it('test eval', async () => {
    await mpz.init(4);

    const output = mpz.testEval(
      {
        bristol: blockTrim(`
          1 3
          2 1 1
          1 1

          2 1 0 1 2 AAdd
        `),
        info: {
          input_name_to_wire_index: {
            a: 0,
            b: 1,
          },
          constants: {},
          output_name_to_wire_index: {
            c: 2,
          },
        },
      },
      {
        a: 3,
        b: 5,
      },
    );

    expect(output).to.deep.eq({ c: 8 });
  });
});
