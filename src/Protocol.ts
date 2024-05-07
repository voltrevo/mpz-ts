import Circuit from "./Circuit";
import Session from "./Session";

export default class Protocol {
  constructor(_circuit: Circuit, _parties: Record<string, string[]>) {}

  join(
    _name: string,
    _input: Record<string, unknown>,
    _send: (to: string, msg: Uint8Array) => void,
  ): Session {
    throw new Error('Not implemented');
  }
}
