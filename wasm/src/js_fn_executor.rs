use js_sys::Function;
use std::sync::mpsc::{self, RecvError, Sender};
use std::sync::Arc;

pub struct JsFnExecutor {
    sender: Sender<Box<dyn FnOnce(&Arc<Function>) + Send>>,
}

impl JsFnExecutor {
    pub fn new(js_func: Arc<Function>) -> Self {
        let (sender, receiver) = mpsc::channel::<Box<dyn FnOnce(&Arc<Function>) + Send>>();

        // Spawn a thread that runs on the main thread to execute the function
        wasm_bindgen_futures::spawn_local(async move {
            while let Ok(task) = receiver.recv() {
                // Execute the task on the main thread
                task(&js_func);
            }
        });

        JsFnExecutor { sender }
    }

    // This method can be called from any thread
    pub fn execute<F, T>(&self, f: F) -> Result<T, RecvError>
    where
        F: FnOnce(&Arc<Function>)->T + Send + 'static,
        T: Send + 'static,
    {
        let (response_sender, response_receiver) = mpsc::channel::<T>();

        self.sender
            .send(Box::new(move |js_func| {
                let response = f(js_func);
                response_sender.send(response).unwrap();
            }))
            .unwrap();

        response_receiver.recv()
    }
}
