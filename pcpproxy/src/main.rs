#![warn(clippy::all)]

mod core;
mod features;

use std::num::NonZeroU16;

use anyhow::Result;
use clap::{Arg, Command};

use crate::core::listen::listen;

#[tokio::main]
async fn main() -> Result<()> {
    if cfg!(debug_assertions) {
        env_logger::Builder::from_default_env()
            .filter_module("pcpproxy", log::LevelFilter::Trace)
            .target(env_logger::Target::Stderr)
            .init();
    }
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::new("listen_port")
                .help("Listen port")
                .required(true)
                .validator(|arg| {
                    NonZeroU16::new(arg.parse()?).ok_or_else(|| anyhow::anyhow!("Zero"))
                }),
        )
        .arg(
            Arg::new("hostname_from_real_server")
                .help("Hostname from real PeerCast (without tcp port)")
                .required(true),
        )
        .arg(
            Arg::new("real_server_host")
                .help("Real PeerCast host (hostname:port)")
                .required(true),
        )
        .get_matches();

    listen(
        NonZeroU16::new(matches.value_of("listen_port").unwrap().parse().unwrap()).unwrap(),
        matches.value_of("hostname_from_real_server").unwrap(),
        matches.value_of("real_server_host").unwrap(),
    )
    .await;
    Ok(())
}
