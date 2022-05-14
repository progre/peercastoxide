use std::{fmt::Debug, sync::Weak};

use async_trait::async_trait;
use serde::Serialize;
use tauri::{generate_context, AppHandle, Invoke, Manager};

#[async_trait]
pub trait WindowDelegate {
    async fn on_command(&self, command: &str) -> Option<&dyn erased_serde::Serialize>;
}

type DynSendSyncWindowDelegate = dyn Send + Sync + WindowDelegate;

fn build_app(delegate: Weak<DynSendSyncWindowDelegate>) -> tauri::App {
    tauri::Builder::default()
        .manage(delegate)
        .invoke_handler(|Invoke { message, resolver }| {
            tauri::async_runtime::spawn(async move {
                if let Some(delegate) = message
                    .state_ref()
                    .get::<Weak<DynSendSyncWindowDelegate>>()
                    .upgrade()
                {
                    resolver.resolve(delegate.on_command(message.command()).await);
                }
            });
        })
        .build(generate_context!())
        .map_err(|err| {
            let mut note = "";
            if let tauri::Error::Runtime(tauri_runtime::Error::CreateWebview(err)) = &err {
                if err.to_string().contains("WebView2") {
                    note = concat!(
                        "WebView2ランタイムをインストールすると",
                        "このエラーが解決する可能性があります。"
                    );
                }
            }
            let none: Option<&tauri::Window> = None;
            tauri::api::dialog::blocking::message(
                none,
                "Fatal",
                &format!("アプリケーションの起動に失敗しました。{}({}) ", note, err),
            );
            err
        })
        .expect("error while running tauri application")
}

pub struct Window {
    app_handle: std::sync::Mutex<Option<AppHandle>>,
}

impl Window {
    pub fn new() -> Self {
        Self {
            app_handle: Default::default(),
        }
    }

    pub fn run(&self, delegate: Weak<DynSendSyncWindowDelegate>) {
        let app = build_app(delegate);
        *self.app_handle.lock().unwrap() = Some(app.app_handle());
        app.run(|_, _| {});
    }

    pub fn emit_all<S: Serialize + Clone + Debug>(&self, event: &str, payload: S) {
        if let Some(app_handle) = self.app_handle.lock().unwrap().as_ref() {
            app_handle.emit_all(event, payload).unwrap();
        }
    }
}
