use std::num::NonZeroU16;

use getset::{Getters, Setters};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Getters, Serialize, Setters)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    #[getset(get = "pub", set = "pub")]
    real_server_host: String,
    #[getset(get = "pub", set = "pub")]
    ipv4_addr_from_real_server: String,
    #[getset(get = "pub", set = "pub")]
    ipv4_port: NonZeroU16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            real_server_host: "localhost:7145".into(),
            ipv4_addr_from_real_server: "127.0.0.1".into(),
            ipv4_port: 7144.try_into().unwrap(),
        }
    }
}
