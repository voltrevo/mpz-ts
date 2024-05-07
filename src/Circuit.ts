export default class Circuit {
  eval(_input: Record<string, unknown>): Record<string, unknown> {
    throw new Error('Not implemented');
  }

  static fromCircom(_files: Record<string, string>): Circuit {
    throw new Error('Not implemented');
  }
}
