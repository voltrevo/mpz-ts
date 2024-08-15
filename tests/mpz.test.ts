import { expect } from 'chai';
import * as mpz from '../src';

describe('mpz', () => {
  it('test fn', async () => {
    await mpz.init(4);

    const [aliceComms, bobComms] = makeCommsPair();

    const responses = await Promise.all([
      mpz.testAlice(aliceComms.send, aliceComms.recv),
      mpz.testBob(bobComms.send, bobComms.recv),
    ]);

    console.log({ responses });

    // expect(msg).to.deep.eq(Uint8Array.from([0, 0, 0, 4, 2, 0, 0, 0]));
  });
});

type Comms = {
  send(data: Uint8Array): void;

  recvBuf: CommsBuf;
  recv(maxLen: number): Uint8Array;
};

function makeCommsPair(): [Comms, Comms] {
  const a = { recvBuf: new CommsBuf('a') } as Comms;
  const b = { recvBuf: new CommsBuf('b') } as Comms;

  a.send = data => b.recvBuf.push(data);
  a.recv = maxLen => a.recvBuf.pop(maxLen);

  b.send = data => a.recvBuf.push(data);
  b.recv = maxLen => b.recvBuf.pop(maxLen);

  return [a, b];
}

class CommsBuf {
  buf = new Uint8Array(1024);
  bufLen = 0;

  constructor(public name: string) {}

  push(data: Uint8Array) {
    while (data.length + this.bufLen > this.buf.length) {
      const newBuf = new Uint8Array(2 * this.buf.length);
      newBuf.set(this.buf);
      this.buf = newBuf;
    }

    this.buf.set(data, this.bufLen);
    this.bufLen += data.length;
  }

  pop(maxLen: number) {
    if (maxLen >= this.bufLen) {
      const res = this.buf.subarray(0, this.bufLen);
      this.buf = new Uint8Array(1024);
      this.bufLen = 0;

      return res;
    }

    const res = new Uint8Array(maxLen);
    res.set(this.buf.subarray(0, maxLen));

    this.buf.copyWithin(0, maxLen, this.bufLen);
    this.bufLen -= maxLen;

    return res;
  }
}
