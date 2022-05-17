use std::{process::Stdio, sync::Weak};

use serde_json::Value;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    spawn,
    task::JoinHandle,
};

pub trait SubProcessDelegate {
    fn on_json(&self, json: &Value);
}

pub struct SubProcess {}

impl SubProcess {
    pub fn run(delegate: Weak<dyn Send + Sync + SubProcessDelegate>) -> JoinHandle<()> {
        let mut child = Command::new("pcpproxy")
            .arg("127.0.0.1:7144")
            .arg("127.0.0.1:7264")
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        println!("child");
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        spawn(async move {
            loop {
                let line = lines.next_line().await.unwrap().unwrap();
                let json: Value = serde_json::from_str(&line).unwrap();
                if let Some(delegate) = delegate.upgrade() {
                    delegate.on_json(&json);
                }
            }
        })
    }
}
