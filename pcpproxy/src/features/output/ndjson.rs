use serde_json::{json, Value};

use crate::features::pcp::atom::Atom;

pub struct NDJson {
    client_host: String,
    server_host: String,
    upload: bool,
}

impl NDJson {
    pub fn upload(client_host: String, server_host: String) -> Self {
        Self {
            client_host,
            server_host,
            upload: true,
        }
    }

    pub fn download(client_host: String, server_host: String) -> Self {
        Self {
            client_host,
            server_host,
            upload: false,
        }
    }

    pub fn output_raw(&self, payload: &str) {
        self.output_internal("raw", json!(payload));
    }

    pub fn output(&self, atom: &Atom) {
        self.output_internal("atom", json!(atom));
    }

    fn output_internal(&self, type_param: &str, payload: Value) {
        let direction = if self.upload { "upload" } else { "download" };
        println!(
            "{}",
            json!({
                "clientHost": self.client_host,
                "serverHost": self.server_host,
                "direction": direction,
                "type": type_param,
                "payload": payload,
            })
        );
    }
}
