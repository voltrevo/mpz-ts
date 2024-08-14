mod js_fn_executor;
mod setup_garble;
mod js_conn;

use std::time::Duration;

use console_error_panic_hook::set_once as set_panic_hook;
use js_conn::JsConn;
use mpz_circuits::circuits::AES128;
use mpz_common::executor::STExecutor;
use mpz_garble::{DecodePrivate, Execute, Memory};
use serio::{codec::{Bincode, Codec}, SinkExt, StreamExt};
use setup_garble::Role;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::console;
use gloo_timers::future::sleep;

#[wasm_bindgen]
pub fn init_ext() {
    set_panic_hook();
}

#[wasm_bindgen]
pub async fn start_async_task() {
    // An example async task
    let future_task = async {
        // Do some asynchronous work here
        console::log_1(&"Task is running...".into());
    };

    // Spawn the future on the local thread
    spawn_local(future_task);

    sleep(Duration::from_millis(100)).await;
    // Delay::new(Duration::from_millis(100)).await.unwrap();

    console::log_1(&"Here".into());
}

#[wasm_bindgen]
pub async fn test_send(
    send: &js_sys::Function,
    recv: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    console::log_1(&"test_send".into());

    let conn = JsConn::new(send, recv);
    let mut channel = Bincode.new_framed(conn);

    channel.send("Hi".to_string()).await.unwrap();

    Ok(JsValue::UNDEFINED)
}

#[wasm_bindgen]
pub async fn test_recv(
    send: &js_sys::Function,
    recv: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    console::log_1(&"test_recv".into());

    let conn = JsConn::new(send, recv);
    let mut channel = Bincode.new_framed(conn);

    let msg = channel.next::<String>().await.unwrap().unwrap();

    Ok(msg.into())
}

#[wasm_bindgen]
pub async fn test_alice(
    send: &js_sys::Function,
    recv: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    console::log_1(&"test".into());

    let conn = JsConn::new(send, recv);
    let channel = Bincode.new_framed(conn);

    // Create an executor and use it to instantiate a vm for garbled circuits.
    let executor = STExecutor::new(channel);
    console::log_1(&"test2".into());
    let mut garble_vm = setup_garble::setup_garble(Role::Alice, executor, 256)
        .await
        .unwrap();
    console::log_1(&"test3".into());

    // Define input and output types.
    let key = garble_vm.new_private_input::<[u8; 16]>("key").unwrap();
    let msg = garble_vm.new_blind_input::<[u8; 16]>("msg").unwrap();
    let ciphertext = garble_vm.new_output::<[u8; 16]>("ciphertext").unwrap();

    // Assign the key.
    garble_vm
        .assign(
            &key,
            [
                0x2b_u8, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 0xab, 0xf7, 0x15, 0x88, 0x09,
                0xcf, 0x4f, 0x3c,
            ],
        )
        .unwrap();

    // Load the AES circuit.
    let circuit = AES128.clone();

    // Execute the circuit.
    garble_vm
        .execute(circuit, &[key, msg], &[ciphertext.clone()])
        .await
        .unwrap();

    // Receive output information from Bob.
    let mut output = garble_vm.decode_private(&[ciphertext]).await.unwrap();

    // Print the encrypted text.
    let encrypted: [u8; 16] = output.pop().unwrap().try_into().unwrap();

    Ok(format!("Encrypted text is {:x?}", encrypted).into())
}

#[wasm_bindgen]
pub async fn test_bob(
    send: &js_sys::Function,
    recv: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    let conn = JsConn::new(send, recv);
    let channel = Bincode.new_framed(conn);

    // Create an executor and use it to instantiate a vm for garbled circuits.
    let executor = STExecutor::new(channel);
    let mut garble_vm = setup_garble::setup_garble(Role::Bob, executor, 256).await.unwrap();

    // Define input and output types.
    let key = garble_vm.new_blind_input::<[u8; 16]>("key").unwrap();
    let msg = garble_vm.new_private_input::<[u8; 16]>("msg").unwrap();
    let ciphertext = garble_vm.new_output::<[u8; 16]>("ciphertext").unwrap();

    // Assign the message.
    garble_vm
        .assign(
            &msg,
            [
                0x6b_u8, 0xc1, 0xbe, 0xe2, 0x2e, 0x40, 0x9f, 0x96, 0xe9, 0x3d, 0x7e, 0x11, 0x73,
                0x93, 0x17, 0x2a,
            ],
        )
        .unwrap();

    // Load the AES circuit.
    let circuit = AES128.clone();

    // Execute the circuit.
    garble_vm
        .execute(circuit, &[key, msg], &[ciphertext.clone()])
        .await
        .unwrap();

    // Send output information to Alice.
    garble_vm.decode_blind(&[ciphertext]).await.unwrap();

    Ok(JsValue::UNDEFINED)
}
