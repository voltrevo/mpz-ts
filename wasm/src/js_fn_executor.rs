use gloo_timers::future::sleep;
use js_sys::Function;
use std::sync::mpsc::{self, Sender, TryRecvError};
use std::sync::Arc;
use std::time::Duration;

pub struct JsFnExecutor {
    sender: Sender<Box<dyn FnOnce(&Arc<Function>) + Send>>,
}

impl JsFnExecutor {
    pub fn new(js_func: Arc<Function>) -> Self {
        let (sender, receiver) = mpsc::channel::<Box<dyn FnOnce(&Arc<Function>) + Send>>();

        // Spawn a thread that runs on the main thread to execute the function
        wasm_bindgen_futures::spawn_local(async move {
            loop {
                match receiver.try_recv() {
                    Ok(task) => {
                        // Execute the task on the main thread
                        task(&js_func);
                    },
                    Err(TryRecvError::Empty) => {
                        sleep(Duration::from_millis(10)).await;
                    },
                    Err(TryRecvError::Disconnected) => {
                        break;
                    }
                }
            }
        });

        JsFnExecutor { sender }
    }

    // This method can be called from any thread
    pub fn execute<F, T>(&self, f: F) -> mpsc::Receiver<T>
    where
        F: FnOnce(&Arc<Function>)->T + Send + 'static,
        T: Send + 'static,
    {
        let (response_sender, response_receiver) = mpsc::channel::<T>();

        self.sender
            .send(Box::new(move |js_func| {
                let response = f(js_func);
                let _ = response_sender.send(response);
            }))
            .unwrap();

        response_receiver
    }
}
