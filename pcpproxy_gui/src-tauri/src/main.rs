#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod core;
mod features;

use std::sync::{Arc, Weak};

use async_trait::async_trait;
use getset::Setters;
use serde_json::{json, Value};
use tokio::sync::Mutex;

use crate::core::settings::Settings;
use crate::features::{
    files::settings::{
        load_settings_and_show_dialog_if_error, save_settings_and_show_dialog_if_error,
    },
    process::sub_process::{SubProcess, SubProcessDelegate},
    ui::window::{Window, WindowDelegate},
};

#[derive(Setters)]
struct SubProcessDelegateImpl {
    #[getset(set = "pub")]
    window: Weak<Window>,
}

impl SubProcessDelegateImpl {
    pub fn new() -> Self {
        let weak = Default::default();
        Self { window: weak }
    }
}

impl SubProcessDelegate for SubProcessDelegateImpl {
    fn on_json(&self, json: &serde_json::Value) {
        if let Some(window) = self.window.upgrade() {
            window.emit_all("json", json);
        }
    }
}

#[derive(Setters)]
struct WindowDelegateImpl {
    settings: std::sync::Mutex<Settings>,
    #[getset(set = "pub")]
    sub_process: Weak<Mutex<SubProcess>>,
}

impl WindowDelegateImpl {
    pub fn new(settings: Settings) -> Self {
        let weak = Default::default();
        Self {
            settings: std::sync::Mutex::new(settings),
            sub_process: weak,
        }
    }
}

#[async_trait]
impl WindowDelegate for WindowDelegateImpl {
    async fn on_command(&self, command: &str, payload: &Value) -> Option<Value> {
        match command {
            "initial_data" => {
                let settings = self.settings.lock().unwrap();
                Some(json!({ "settings": serde_json::to_value::<&Settings>(&settings).unwrap() }))
            }
            "set_settings" => {
                log::trace!("payload {:?}", payload);
                let settings: Settings = serde_json::from_value(payload.clone()).unwrap();
                save_settings_and_show_dialog_if_error(&settings).await;
                None
            }
            "restart" => {
                let settings = self.settings.lock().unwrap().clone();
                self.sub_process
                    .upgrade()
                    .unwrap()
                    .lock()
                    .await
                    .spawn(&settings)
                    .await;
                None
            }
            _ => unreachable!(),
        }
    }
}

#[tokio::main]
async fn main() {
    if cfg!(debug_assertions) {
        env_logger::Builder::from_default_env()
            .filter_module("app", log::LevelFilter::Trace)
            .init();
        log::trace!("log");
    }

    let settings = load_settings_and_show_dialog_if_error().await;

    let mut sub_process_delegate = SubProcessDelegateImpl::new();
    let mut window_delegate = WindowDelegateImpl::new(settings.clone());

    let sub_process = Default::default();
    let window = Arc::new(Window::new());

    sub_process_delegate.set_window(Arc::downgrade(&window));
    window_delegate.set_sub_process(Arc::downgrade(&sub_process));

    let sub_process_delegate = Arc::new(sub_process_delegate);
    let window_delegate = Arc::new(window_delegate);

    {
        let mut sub_process = sub_process.lock().await;
        sub_process.set_delegate(Arc::<SubProcessDelegateImpl>::downgrade(
            &sub_process_delegate,
        ));

        sub_process.spawn(&settings).await;
    }
    window.run(Arc::<WindowDelegateImpl>::downgrade(&window_delegate));
}
