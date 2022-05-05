use std::collections::HashMap;
use std::sync::Arc;
use std::{net::IpAddr, time::Instant};

use anyhow::Result;
use log::*;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::RwLock;
use tokio::{io::AsyncReadExt, sync::Mutex};

use crate::core::legacy::console::console_utils::proxy_message;
use crate::core::legacy::console::{console_color::ConsoleColor, printer::HttpPrinter};

use super::pipe::big_vec;
use super::pipe::pipe_pcp;

use super::sub_servers::SubServers;

#[derive(Error, Debug)]
#[error("host not found error")]
pub struct HostNotFoundError {}

async fn pipe_for_get_channel(
    mut from: OwnedReadHalf,
    mut to: OwnedWriteHalf,
    sub_servers: &Mutex<SubServers>,
    color: ConsoleColor,
) -> Result<()> {
    let mut printer = HttpPrinter::new(&color);
    let mut buf = big_vec(1024 * 1024);
    loop {
        let n = from.read(&mut buf).await?;
        to.write_all(&buf[0..n]).await?;
        let text = String::from_utf8(buf[0..n].to_vec()).unwrap();
        printer.print(&text);
        if text.replace('\r', "").ends_with("\n\n") {
            break;
        }
        trace!("loop");
    }
    printer.print_eos();
    pipe_pcp(from, to, sub_servers, color).await
}

pub async fn proxy_for_get_channel(
    incoming_stream: TcpStream,
    channel_id: String,
    id_table: Arc<RwLock<HashMap<String, (String, Instant)>>>,
) -> Result<()> {
    let local_addr = if let IpAddr::V4(local_addr) = incoming_stream.local_addr().unwrap().ip() {
        local_addr
    } else {
        unreachable!();
    };
    let incoming_addr = incoming_stream.peer_addr().unwrap();

    let (incoming_read, incoming_write) = incoming_stream.into_split();
    let client = id_table
        .write()
        .await
        .get(&channel_id.to_uppercase())
        .ok_or(HostNotFoundError {})?
        .0
        .clone();
    let client_socket = TcpStream::connect(&client).await?;
    let (client_read, client_write) = client_socket.into_split();

    let incoming_color = ConsoleColor::random_color(&format!("{}_{}", incoming_addr, channel_id));
    let client_color = ConsoleColor::random_color(&format!("{}_{}", &client, channel_id));

    println!(
        "Accept {} (channel_id: {})",
        proxy_message(
            &incoming_addr.to_string(),
            &incoming_color,
            &local_addr.to_string(),
            &client,
            &client_color
        ),
        channel_id,
    );

    let sub_servers = Arc::new(Mutex::new(SubServers::new(local_addr)));
    {
        let sub_servers = sub_servers.clone();
        spawn(async move {
            pipe_for_get_channel(
                incoming_read,
                client_write,
                sub_servers.as_ref(),
                incoming_color,
            )
            .await
        });
    }
    {
        spawn(async move {
            pipe_for_get_channel(
                client_read,
                incoming_write,
                sub_servers.as_ref(),
                client_color,
            )
            .await
        });
    }
    Ok(())
}
