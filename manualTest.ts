import * as mpz from './src';

async function main() {
  await mpz.init();

  const [aliceComms, bobComms] = makeCommsPair();

  console.log('here');

  const responses = await Promise.all([
    mpz.testAlice(aliceComms.send, aliceComms.recv),
    mpz.testBob(bobComms.send, bobComms.recv),
  ]);

  console.log({ responses });
}

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
    console.log(this.name, 'push', data);

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

      if (res.length > 0) {
        console.log(this.name, 'pop', res);
      }
      return res;
    }

    const res = new Uint8Array(maxLen);
    res.set(this.buf.subarray(0, maxLen));

    this.buf.copyWithin(0, maxLen, this.bufLen);
    this.bufLen -= maxLen;

    console.log(this.name, 'pop', res);
    return res;
  }
}

main().catch(console.error);
