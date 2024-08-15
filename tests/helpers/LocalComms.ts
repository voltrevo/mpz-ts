export type LocalComms = {
  send(data: Uint8Array): void;

  recvBuf: LocalCommsBuf;
  recv(): Uint8Array;
};

export function makeLocalCommsPair(): [LocalComms, LocalComms] {
  const a = { recvBuf: new LocalCommsBuf() } as LocalComms;
  const b = { recvBuf: new LocalCommsBuf() } as LocalComms;

  a.send = data => b.recvBuf.push(data);
  a.recv = () => a.recvBuf.pop();

  b.send = data => a.recvBuf.push(data);
  b.recv = () => b.recvBuf.pop();

  return [a, b];
}

export class LocalCommsBuf {
  buf = new Uint8Array(1024);
  bufLen = 0;

  constructor() {}

  push(data: Uint8Array) {
    while (data.length + this.bufLen > this.buf.length) {
      const newBuf = new Uint8Array(2 * this.buf.length);
      newBuf.set(this.buf);
      this.buf = newBuf;
    }

    this.buf.set(data, this.bufLen);
    this.bufLen += data.length;
  }

  pop() {
    const res = this.buf.slice(0, this.bufLen);
    this.bufLen = 0;

    return res;
  }
}
