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
            Arg::new("ip_address_from_peer_cast")
                .help("Ip address from PeerCast (host:port)")
                .required(true),
        )
        .arg(
            Arg::new("peer_cast_host")
                .help("PeerCast host (host:port)")
                .required(true),
        )
        .get_matches();

    log4rs::init_file("log4rs.yml", Default::default()).unwrap_or_default();

    listen(
        matches.value_of("ip_address_from_peer_cast").unwrap(),
        matches.value_of("peer_cast_host").unwrap(),
    )
    .await
}
