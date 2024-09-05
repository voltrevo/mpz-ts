import { BackendSession, Circuit, MpcSettings } from "mpc-framework-common";
import AsyncQueue from "./AsyncQueue";
import defer from "./defer";
import { Keccak } from "sha3";
import { pack } from "msgpackr";
import buffersEqual from "./buffersEqual";

export default class MpzDeapFollowerSession implements BackendSession {
  peerName: string;
  msgQueue = new AsyncQueue<Uint8Array>();
  result = defer<Record<string, unknown>>();

  constructor(
    public circuit: Circuit,
    public mpcSettings: MpcSettings,
    public input: Record<string, unknown>,
    public send: (to: string, msg: Uint8Array) => void,
  ) {
    this.peerName = mpcSettings[0].name ?? "0";

    this.run().catch(err => {
      this.result.reject(err);
    });
  }

  handleMessage(from: string, msg: Uint8Array): void {
    if (from !== this.peerName) {
      console.error("Received message from unknown peer", from);
      return;
    }

    this.msgQueue.push(msg);
  }

  async run() {
    const setupHash = new Keccak().update(
      pack([this.circuit, this.mpcSettings])
    ).digest();

    this.send(this.peerName, setupHash);

    const msg = await this.msgQueue.pop();

    if (!buffersEqual(msg, setupHash)) {
      throw new Error("Setup hash mismatch: check peer settings match");
    }

    throw new Error("todo: implement me");
  }

  output(): Promise<Record<string, unknown>> {
    return this.result.promise;
  }
}
