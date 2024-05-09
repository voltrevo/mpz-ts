# mpz-ts

Experimental TypeScript library for building MPC apps backed by
[privacy-scaling-explorations/mpz](https://github.com/privacy-scaling-explorations/mpz).

## Status

This is an **UNIMPLEMENTED** stub module. It exposes the API we are aiming to
support. It does not work yet.

## Usage

```sh
npm install circuit-2-arithc mpz-ts
```

```ts
import * as c2a from 'circuit-2-arithc';
import * as mpz from 'mpz-ts';

const circuitSrc = {
  // In a real project you should be able to include these as regular files, but
  // how those files find their way into this format depends on your build tool.

  'main.circom': `
    pragma circom 2.0.0;

    template Adder() {
        signal input a, b;
        signal output c;

        c <== a + b;
    }

    component main = Adder();
  `,
};

const circuit = c2a.Circuit.compile(circuitSrc);

console.log(
  circuit.eval({
    a: 3,
    b: 5,
  }),
); // { c: 8 }

const parties = {
  alice: ['a'],
  bob: ['b'],
};

const protocol = new mpz.Protocol(circuit.toMpzCircuit(), parties);

function send(to: string, msg: Uint8Array) {
  // implement sending a message to the specified party
}

const session = protocol.join('alice', { a: 3 }, send);

// This is just a hypothetical API for getting external messages
onMessageReceived((from: string, msg: Uint8Array) => {
  // The important part is that you provide the messages to the session like
  // this
  session.handleMessage(from, msg);
});

// assume someone else joins as bob and provides { b: 5 }

console.log(await session.output()); // { c: 8 }
```

## Example Project

For a more complete example in the form of a repository using `mpz-ts`, see
[mpz-ts-example](https://github.com/voltrevo/mpz-ts-example).
