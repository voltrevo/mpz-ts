export default class Session {
  handleMessage(_from: string, _msg: Uint8Array) {
    throw new Error('Not implemented');
  }

  async output(): Promise<Record<string, unknown>> {
    throw new Error('Not implemented');
  }
}
