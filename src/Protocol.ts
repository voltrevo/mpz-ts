import Circuit from "./Circuit";
import Session from "./Session";

export type MpcParticipantSettings = {
  name?: string,
  inputs: string[],
  outputs: string[],
};

export type MpcSettings = MpcParticipantSettings[];

export default class Protocol {
  constructor(_circuit: Circuit, _mpcSettings: MpcSettings) {}

  join(
    _name: string,
    _input: Record<string, unknown>,
    _send: (to: string, msg: Uint8Array) => void,
  ): Session {
    throw new Error('Not implemented');
  }
}
