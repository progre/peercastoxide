#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod features;

use std::sync::{Arc, Weak};

use async_trait::async_trait;
use features::{
    process::subprocess::{SubProcess, SubProcessDelegate},
    ui::window::{Window, WindowDelegate},
};

struct WindowDelegateImpl {}

#[async_trait]
impl WindowDelegate for WindowDelegateImpl {
    async fn on_command(&self, command: &str) -> Option<&dyn erased_serde::Serialize> {
        println!("{}", command);
        None
    }
}

struct SubProcessDelegateImpl {
    window: Weak<Window>,
}

impl SubProcessDelegate for SubProcessDelegateImpl {
    fn on_json(&self, json: &serde_json::Value) {
        if let Some(window) = self.window.upgrade() {
            window.emit_all("json", json);
        }
    }
}

#[tokio::main]
async fn main() {
    let window = Arc::new(Window::new());
    let window_delegate = Arc::new(WindowDelegateImpl {});
    let sub_process_delegate = Arc::new(SubProcessDelegateImpl {
        window: Arc::downgrade(&window),
    });
    let weak = Arc::downgrade(&sub_process_delegate);
    SubProcess::run(weak);
    let weak = Arc::downgrade(&window_delegate);
    window.run(weak);
}
