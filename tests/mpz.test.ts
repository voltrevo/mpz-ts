import { expect } from "chai";
import * as mpz from '../src';

describe('mpz', () => {
  it('test fn', async () => {
    await mpz.init();

    expect(mpz.test('foo')).to.eq('foo');
  });
});
