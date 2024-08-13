import { expect } from "chai";
import * as mpz from '../src';

describe('mpz', () => {
  it('test fn', async () => {
    await mpz.init();

    let msg = new Uint8Array();

    await mpz.test(
      m => { msg = m; },
      () => { throw new Error('boom'); },
    );

    expect(msg).to.deep.eq(Uint8Array.from([0, 0, 0, 4, 2, 0, 0, 0]));
  });
});
