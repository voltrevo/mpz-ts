use std::{cmp::min, sync::{mpsc::Receiver, Arc, Mutex}};

use futures::{AsyncRead, AsyncWrite};
use wasm_bindgen::JsValue;
use web_sys::console;

use crate::js_fn_executor::JsFnExecutor;

pub struct JsConn {
    pub send: JsFnExecutor,
    pub recv: JsFnExecutor,
    pub buf_receivers: Vec<Arc<Mutex<Receiver<Vec<u8>>>>>,
    pub recv_buf: Vec<u8>,
    pub count: usize,
    pub id: char,
}

impl JsConn {
    pub fn new(
        send: &js_sys::Function,
        recv: &js_sys::Function,
        id: char,
    ) -> Self {
        Self {
            send: JsFnExecutor::new(Arc::new(send.clone())),
            recv: JsFnExecutor::new(Arc::new(recv.clone())),
            buf_receivers: vec![],
            recv_buf: vec![],
            count: 0,
            id,
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

        if !self.buf_receivers.is_empty() {
            console::log_1(&format!("{}: {} buf receivers left", self.id, self.buf_receivers.len()).into());
        }
    }
}

impl AsyncWrite for JsConn {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        console::log_1(&format!("{}: write {:?}", self.id, buf).into());

        let len = buf.len();
        let buf = Arc::new(Vec::from_iter(buf.iter().cloned()));
        let id = self.id;

        self.send.execute(move |send| {
            console::log_1(&format!("{}: calling js send", id).into());
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
        console::log_1(&format!("{}: flush", self.id).into());
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
        if self.count < 10 {
            console::log_1(&format!("{}: poll_read {}", self.id, self.count).into());
        }

        self.as_mut().count += 1;

        self.try_empty_buf_receivers();

        if !self.recv_buf.is_empty() {
            let len = min(buf.len(), self.recv_buf.len());
            buf[0..len].copy_from_slice(&self.recv_buf[0..len]);
            self.recv_buf.drain(0..len);

            console::log_1(&format!("{}: Received {} bytes", self.id, len).into());
            self.as_mut().count = 0;

            return std::task::Poll::Ready(Ok(len));
        }

        let send_len = buf.len();

        let waker = cx.waker().clone();

        let buf_receiver = self.recv.execute(move |recv| {
            let recv_bytes = recv.call1(&JsValue::UNDEFINED, &send_len.into()).unwrap();

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
