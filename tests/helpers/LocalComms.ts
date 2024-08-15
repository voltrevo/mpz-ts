export type LocalComms = {
  send(data: Uint8Array): void;

  recvBuf: LocalCommsBuf;
  recv(maxLen: number): Uint8Array;
};

export function makeLocalCommsPair(): [LocalComms, LocalComms] {
  const a = { recvBuf: new LocalCommsBuf() } as LocalComms;
  const b = { recvBuf: new LocalCommsBuf() } as LocalComms;

  a.send = data => b.recvBuf.push(data);
  a.recv = maxLen => a.recvBuf.pop(maxLen);

  b.send = data => a.recvBuf.push(data);
  b.recv = maxLen => b.recvBuf.pop(maxLen);

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
