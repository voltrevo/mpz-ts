import { BackendSession, Circuit, MpcSettings } from "mpc-framework-common";
import AsyncQueue from "./AsyncQueue";
import defer from "./defer";

export default class MpzDeapLeaderSession implements BackendSession {
  peerName: string;
  msgQueue = new AsyncQueue<Uint8Array>();
  result = defer<Record<string, unknown>>();

  constructor(
    public circuit: Circuit,
    public mpcSettings: MpcSettings,
    public input: Record<string, unknown>,
    public send: (to: string, msg: Uint8Array) => void,
  ) {
    this.peerName = mpcSettings[1].name ?? "1";

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
    throw new Error("todo: implement me");
  }

  output(): Promise<Record<string, unknown>> {
    return this.result.promise;
  }
}
