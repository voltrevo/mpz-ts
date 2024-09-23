import { Backend, BackendSession, checkSettingsValid, Circuit, MpcSettings } from "mpc-framework-common";
import MpzSemiHonestSession from "./MpzSemiHonestSession";

export default class MpzSemiHonestBackend implements Backend {
  run(
    circuit: Circuit,
    mpcSettings: MpcSettings,
    name: string,
    input: Record<string, unknown>,
    send: (to: string, msg: Uint8Array) => void,
  ): BackendSession {
    const checkResult = (
      checkSettingsValid(circuit, mpcSettings, name, input) ??
      checkSettingsValidForMpzSemiHonest(circuit, mpcSettings)
    );

    if (checkResult !== undefined) {
      throw checkResult;
    }

    const leaderName = mpcSettings[0].name ?? "0";
    const followerName = mpcSettings[1].name ?? "1";

    if (name !== leaderName && name !== followerName) {
      throw new Error(`Unknown participant name: ${name}`);
    }

    return new MpzSemiHonestSession(
      circuit,
      mpcSettings,
      input,
      send,
      name === leaderName,
    );
  }
}

export function checkSettingsValidForMpzSemiHonest(
  circuit: Circuit,
  mpcSettings: MpcSettings,
): Error | undefined {
  if (mpcSettings.length !== 2) {
    return new Error("MpzSemiHonestBackend requires exactly two participants");
  }

  for (const participant of mpcSettings) {
    if (!checkStringSetsEqual(
      participant.outputs,
      Object.keys(circuit.info.output_name_to_wire_index)
    )) {
      return new Error(
        "Participant outputs do not match the circuit",
      );
    }

    // Note: It's also possible for the follower to get no outputs, but this is
    // not currently supported here.
  }

  return undefined;
}

function checkStringSetsEqual(a: string[], b: string[]) {
  const setA = new Set(a);
  const setB = new Set(b);

  if (setA.size !== setB.size) {
    return false;
  }

  for (const elem of setA) {
    if (!setB.has(elem)) {
      return false;
    }
  }

  return true;
}
