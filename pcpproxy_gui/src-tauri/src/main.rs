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
    sub_process: Weak<std::sync::Mutex<SubProcess>>,
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
                Some(json!({
                    "settings": {
                        "realServerHost": settings.real_server_host(),
                        "ipv4AddrFromRealServer": settings.ipv4_addr_from_real_server(),
                        "ipv4Port": settings.ipv4_port(),
                    }
                }))
            }
            "set_settings" => {
                log::trace!("payload {:?}", payload);
                let obj = payload.as_object().unwrap();
                let settings = {
                    let settings = &mut self.settings.lock().unwrap();
                    settings
                        .set_real_server_host(obj["realServerHost"].as_str().unwrap().to_owned());
                    settings.set_ipv4_addr_from_real_server(
                        obj["ipv4AddrFromRealServer"].as_str().unwrap().to_owned(),
                    );
                    settings.set_ipv4_port(
                        (obj["ipv4Port"].as_u64().unwrap() as u16)
                            .try_into()
                            .unwrap(),
                    );
                    settings.clone()
                };
                save_settings_and_show_dialog_if_error(&settings).await;
                self.sub_process
                    .upgrade()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .spawn(&settings);
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

    let sub_process = Arc::new(std::sync::Mutex::new(SubProcess::new()));
    let window = Arc::new(Window::new());

    sub_process_delegate.set_window(Arc::downgrade(&window));
    window_delegate.set_sub_process(Arc::downgrade(&sub_process));

    let sub_process_delegate = Arc::new(sub_process_delegate);
    let window_delegate = Arc::new(window_delegate);

    {
        let mut sub_process = sub_process.lock().unwrap();
        sub_process.set_delegate(Arc::<SubProcessDelegateImpl>::downgrade(
            &sub_process_delegate,
        ));

        sub_process.spawn(&settings);
    }
    window.run(Arc::<WindowDelegateImpl>::downgrade(&window_delegate));
}
