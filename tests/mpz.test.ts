import { expect } from 'chai';
import * as mpz from '../src';

describe('mpz', () => {
  it.skip('foo', async () => {
    await mpz.init();

    mpz.startAsyncTask();
  });

  it('test fn', async () => {
    await mpz.init();

    const [aliceComms, bobComms] = makeCommsPair();

    console.log('here');

    const responses = await Promise.all([
      mpz.testAlice(aliceComms.send, aliceComms.recv),
      mpz.testBob(bobComms.send, bobComms.recv),
    ]);

    console.log({ responses });

    // expect(msg).to.deep.eq(Uint8Array.from([0, 0, 0, 4, 2, 0, 0, 0]));
  });

  it.skip('test send recv', async () => {
    await mpz.init();

    const [aliceComms, bobComms] = makeCommsPair();

    const responses = await Promise.all([
      mpz.testSend(aliceComms.send, aliceComms.recv),
      mpz.testRecv(bobComms.send, bobComms.recv),
    ]);

    expect(responses).to.deep.eq([undefined, 'Hi']);
  });
});

type Comms = {
  send(data: Uint8Array): void;

  recvBuf: CommsBuf;
  recv(maxLen: number): Uint8Array;
};

function makeCommsPair(): [Comms, Comms] {
  const a = { recvBuf: new CommsBuf() } as Comms;
  const b = { recvBuf: new CommsBuf() } as Comms;

  a.send = data => b.recvBuf.push(data);
  a.recv = maxLen => a.recvBuf.pop(maxLen);

  b.send = data => a.recvBuf.push(data);
  b.recv = maxLen => b.recvBuf.pop(maxLen);

  return [a, b];
}

class CommsBuf {
  buf = new Uint8Array(1024);
  bufLen = 0;

  push(data: Uint8Array) {
    console.log('push', data);

    while (data.length + this.bufLen > this.buf.length) {
      const newBuf = new Uint8Array(2 * this.buf.length);
      newBuf.set(this.buf);
      this.buf = newBuf;
    }

    this.buf.set(data, this.bufLen);
    this.bufLen += data.length;
  }

  pop(maxLen: number) {
    if (maxLen >= this.buf.length) {
      const res = this.buf.subarray(0, this.bufLen);
      this.buf = new Uint8Array(1024);
      this.bufLen = 0;

      console.log('pop', res);
      return res;
    }

    const res = new Uint8Array(maxLen);
    res.set(this.buf.subarray(0, maxLen));

    this.buf.copyWithin(0, maxLen, this.bufLen);
    this.bufLen -= maxLen;

    console.log('pop', res);
    return res;
  }
}
