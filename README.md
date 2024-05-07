# mpz-ts

Experimental TypeScript library for building MPC apps backed by
[privacy-scaling-explorations/mpz](https://github.com/privacy-scaling-explorations/mpz).

## Status

This is an **UNIMPLEMENTED** stub module. It exposes the API we are aiming to
support. It does not work yet.

## Usage

```sh
npm install mpz
```

```ts
import * as mpz from 'mpz';

const circuitSrc = {
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

const circuit = mpz.Circuit.fromCircom(circuitSrc);

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

const protocol = new mpz.Protocol(circuit, parties);

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
