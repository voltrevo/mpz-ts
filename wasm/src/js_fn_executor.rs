use gloo_timers::future::sleep;
use js_sys::Function;
use web_sys::console;
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
            console::log_1(&"recv loop".into());

            loop {
                console::log_1(&"try_recv".into());
                match receiver.try_recv() {
                    Ok(task) => {
                        console::log_1(&"received task".into());
                        // Execute the task on the main thread
                        task(&js_func);
                        console::log_1(&"finished task".into());
                    },
                    Err(TryRecvError::Empty) => {
                        console::log_1(&"no task received, sleep 10ms".into());
                        sleep(Duration::from_millis(10)).await;
                    },
                    Err(TryRecvError::Disconnected) => {
                        break;
                    }
                }
            }

            console::log_1(&"end recv loop".into());
        });

        console::log_1(&"spawned".into());

        JsFnExecutor { sender }
    }

    // This method can be called from any thread
    pub fn execute<F, T>(&self, f: F) -> mpsc::Receiver<T>
    where
        F: FnOnce(&Arc<Function>)->T + Send + 'static,
        T: Send + 'static,
    {
        console::log_1(&"exec".into());
        let (response_sender, response_receiver) = mpsc::channel::<T>();

        console::log_1(&"sending".into());
        self.sender
            .send(Box::new(move |js_func| {
                console::log_1(&"in task".into());
                let response = f(js_func);
                console::log_1(&"finished task, sending response".into());
                let _ = response_sender.send(response);
                console::log_1(&"response sent".into());
            }))
            .unwrap();
        console::log_1(&"sent".into());

        response_receiver
    }
}
