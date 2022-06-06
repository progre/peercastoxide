use std::{
    process::Stdio,
    sync::{Arc, Weak},
};

use getset::Setters;
use serde_json::Value;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    spawn,
    task::JoinHandle,
};
use windows::Win32::System::Threading::CREATE_NO_WINDOW;

use crate::core::settings::Settings;

pub trait SubProcessDelegate {
    fn on_json(&self, json: &Value);
}

#[derive(Setters)]
pub struct SubProcess {
    #[getset(set = "pub")]
    delegate: Weak<dyn SubProcessDelegate + Send + Sync>,
    process: Option<(Child, JoinHandle<()>)>,
}

impl Default for SubProcess {
    fn default() -> Self {
        Self::new()
    }
}

impl SubProcess {
    pub fn new() -> Self {
        struct EmptySubProcessDelegate {}
        impl SubProcessDelegate for EmptySubProcessDelegate {
            fn on_json(&self, _json: &Value) {}
        }
        let weak = Arc::downgrade(&Arc::new(EmptySubProcessDelegate {}));
        Self {
            delegate: weak,
            process: None,
        }
    }

    pub async fn spawn(&mut self, settings: &Settings) {
        if let Some((process, handle)) = self.process.as_mut() {
            process.kill().await.unwrap();
            handle.abort();
        }
        let mut child = Command::new("pcpproxy")
            .arg(settings.ipv4_port().to_string())
            .arg(settings.ipv4_addr_from_real_server())
            .arg(settings.real_server_host())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .kill_on_drop(true)
            .creation_flags(CREATE_NO_WINDOW.0)
            .spawn()
            .unwrap();
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let delegate = self.delegate.clone();
        self.process = Some((
            child,
            spawn(async move {
                loop {
                    if let Some(line) = lines.next_line().await.unwrap() {
                        let json: Value = serde_json::from_str(&line).unwrap();
                        delegate.upgrade().unwrap().on_json(&json);
                        continue;
                    }
                    break;
                }
            }),
        ));
    }
}
