use std::sync::Arc;

use futures::{AsyncRead, AsyncWrite};
use wasm_bindgen::JsValue;

use crate::js_fn_executor::JsFnExecutor;

pub struct JsConn {
    pub send: JsFnExecutor,
    pub recv: JsFnExecutor,
}

impl JsConn {
    pub fn new(
        send: &js_sys::Function,
        recv: &js_sys::Function,
    ) -> Self {
        Self {
            send: JsFnExecutor::new(Arc::new(send.clone())),
            recv: JsFnExecutor::new(Arc::new(recv.clone())),
        }
    }
}

impl AsyncWrite for JsConn {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let len = buf.len();
        let buf = Arc::new(Vec::from_iter(buf.iter().cloned()));

        self.send.execute(move |send| {
            send.call1(
                &JsValue::UNDEFINED,
                &js_sys::Uint8Array::from(&**buf).into(),
            )
            .unwrap();
        }).unwrap();

        std::task::Poll::Ready(Ok(len))
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
        let send_len = buf.len();

        let exec_result = self.recv.execute(move |recv| {
            let recv_bytes = recv.call1(&JsValue::UNDEFINED, &send_len.into()).unwrap();

            let recv_bytes = js_sys::Uint8Array::from(recv_bytes);
            let recv_len = recv_bytes.length() as usize;

            let mut recv_buf = vec![0u8; recv_len];

            recv_bytes.copy_to(&mut recv_buf);

            recv_buf
        });

        let recv_buf = exec_result.expect("Failed to receive bytes");
        buf.copy_from_slice(&recv_buf);

        std::task::Poll::Ready(Ok(recv_buf.len()))
    }
}
