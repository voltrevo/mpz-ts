import { BackendSession, Circuit, MpcSettings } from "mpc-framework-common";

export default class MpzDeapFollowerSession implements BackendSession {
  constructor(
    circuit: Circuit,
    mpcSettings: MpcSettings,
    input: Record<string, unknown>,
    send: (to: string, msg: Uint8Array) => void,
  ) {
    throw new Error("Method not implemented.");
  }

  handleMessage(from: string, msg: Uint8Array): void {
    throw new Error("Method not implemented.");
  }

  output(): Promise<Record<string, unknown>> {
    throw new Error("Method not implemented.");
  }
}
