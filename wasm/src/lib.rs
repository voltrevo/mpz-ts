use console_error_panic_hook::set_once as set_panic_hook;
use futures::{AsyncRead, AsyncWrite};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use serio::{codec::{Bincode, Codec}, SinkExt};

#[wasm_bindgen]
pub fn init_ext() {
    set_panic_hook();
}

struct JsConn {
    send: js_sys::Function,
    recv: js_sys::Function,
}

impl AsyncWrite for JsConn {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        self.send
            .call1(&JsValue::UNDEFINED, &js_sys::Uint8Array::from(buf).into())
            .unwrap();

        std::task::Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        panic!("Not available") // TODO: Ignore instead?
    }
}

impl AsyncRead for JsConn {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let recv_bytes = self
            .recv
            .call1(&JsValue::UNDEFINED, &buf.len().into())
            .unwrap();

        let recv_bytes = js_sys::Uint8Array::from(recv_bytes);
        let recv_len = recv_bytes.length() as usize;
        recv_bytes.copy_to(&mut buf[0..recv_len]);

        std::task::Poll::Ready(Ok(recv_len))
    }
}

#[derive(Serialize, Deserialize)]
enum TestMessage {
    A, B, C
}

#[wasm_bindgen]
pub async fn test(
    send: &js_sys::Function,
    recv: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    let conn = JsConn {
        send: send.clone(),
        recv: recv.clone(),
    };

    let mut channel = Bincode.new_framed(conn);

    channel.send(TestMessage::C).await.unwrap();

    Ok(JsValue::UNDEFINED)
}
