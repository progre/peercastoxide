#![warn(clippy::all)]

mod core;
mod features;

use anyhow::Result;
use clap::{Arg, Command};

use crate::core::listen::listen;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("host_from_real_server")
                .help("Host from real PeerCast (hostname:port)")
                .required(true),
        )
        .arg(
            Arg::new("real_server_host")
                .help("Real PeerCast host (hostname:port)")
                .required(true),
        )
        .get_matches();

    log4rs::init_file("log4rs.yml", Default::default()).unwrap_or_default();

    listen(
        matches.value_of("host_from_real_server").unwrap(),
        matches.value_of("real_server_host").unwrap(),
    )
    .await
}
