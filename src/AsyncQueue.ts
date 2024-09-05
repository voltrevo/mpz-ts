export default class AsyncQueue<T> {
  private queue: T[] = [];
  private resolvers: ((value: T) => void)[] = [];

  constructor() { }

  push(value: T) {
    if (this.resolvers.length > 0) {
      const resolver = this.resolvers.shift()!;
      resolver(value);
    } else {
      this.queue.push(value);
    }
  }

  async pop(): Promise<T> {
    if (this.queue.length > 0) {
      return this.queue.shift()!;
    }

    return new Promise<T>(resolve => {
      this.resolvers.push(resolve);
    });
  }

  tryPop(): { value: T } | undefined {
    if (this.queue.length > 0) {
      return { value: this.queue.shift()! };
    }

    return undefined
  }
}
