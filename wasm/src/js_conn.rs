use std::{cmp::min, sync::{mpsc::Receiver, Arc, Mutex}};

use futures::{AsyncRead, AsyncWrite};
use wasm_bindgen::JsValue;

use crate::js_fn_executor::JsFnExecutor;

pub struct JsConn {
    pub send: JsFnExecutor,
    pub recv: JsFnExecutor,
    pub buf_receivers: Vec<Arc<Mutex<Receiver<Vec<u8>>>>>,
    pub recv_buf: Vec<u8>,
}

impl JsConn {
    pub fn new(
        send: &js_sys::Function,
        recv: &js_sys::Function,
    ) -> Self {
        Self {
            send: JsFnExecutor::new(Arc::new(send.clone())),
            recv: JsFnExecutor::new(Arc::new(recv.clone())),
            buf_receivers: vec![],
            recv_buf: vec![],
        }
    }

    pub fn try_empty_buf_receivers(&mut self) {
        let mut recv_count = 0usize;
        
        for receiver in self.buf_receivers.iter() {
            if let Ok(r) = receiver.try_lock() {
                if let Ok(recv_buf) = r.try_recv() {
                    self.recv_buf.extend(recv_buf);
                    recv_count += 1;
                    continue;
                }
            }

            break;
        }

        self.buf_receivers.drain(0..recv_count);
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
        });

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
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        self.try_empty_buf_receivers();

        if !self.recv_buf.is_empty() {
            let len = min(buf.len(), self.recv_buf.len());
            buf[0..len].copy_from_slice(&self.recv_buf[0..len]);
            self.recv_buf.drain(0..len);

            return std::task::Poll::Ready(Ok(len));
        }

        let send_len = buf.len();

        let waker = cx.waker().clone();

        let buf_receiver = self.recv.execute(move |recv| {
            let recv_bytes = recv.call0(&JsValue::UNDEFINED).unwrap();

            let recv_bytes = js_sys::Uint8Array::from(recv_bytes);
            let recv_len = recv_bytes.length() as usize;

            let mut recv_buf = vec![0u8; recv_len];
            recv_bytes.copy_to(&mut recv_buf);

            waker.wake();

            recv_buf
        });

        self.buf_receivers.push(Arc::new(Mutex::new(buf_receiver)));

        std::task::Poll::Pending
    }
}
