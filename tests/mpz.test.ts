import { expect } from 'chai';
import * as mpz from '../src';
import { makeLocalCommsPair } from './helpers/LocalComms';

describe('mpz', () => {
  it('test fn', async () => {
    await mpz.init(4);

    const [aliceComms, bobComms] = makeLocalCommsPair();

    const responses = await Promise.all([
      mpz.testAlice(aliceComms.send, aliceComms.recv),
      mpz.testBob(bobComms.send, bobComms.recv),
    ]);

    console.log({ responses });

    // expect(msg).to.deep.eq(Uint8Array.from([0, 0, 0, 4, 2, 0, 0, 0]));
  });
});
