export type Circuit = {
  bristol: string;
  info: {
    input_name_to_wire_index: Record<string, number>;
    constants: Record<string, { value: string; wire_index: number }>;
    output_name_to_wire_index: Record<string, number>;
  };
};

export type MpcParticipantSettings = {
  name?: string,
  inputs: string[],
  outputs: string[],
};

export type MpcSettings = MpcParticipantSettings[];

export type Backend = {
  run(
    circuit: Circuit,
    mpcSettings: MpcSettings,
    name: string,
    input: Record<string, unknown>,
    send: (to: string, msg: Uint8Array) => void,
  ): BackendSession;
};

export type BackendSession = {
  handleMessage(from: string, msg: Uint8Array): void;
  output(): Promise<Record<string, unknown>>;
};
