use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use log::*;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::spawn;
use tokio::sync::RwLock;

use crate::core::legacy::console::{console_color::ConsoleColor, console_utils::proxy_message};

use super::pipe::big_vec;
use super::pipe::pipe;

async fn pipe_for_get_with_tip(
    mut from: OwnedReadHalf,
    mut to: OwnedWriteHalf,
    tip_host: &str,
    ip_address_from_peer_cast: &str,
) -> Result<()> {
    let mut buf = big_vec(1024 * 1024);
    loop {
        let n = from.read(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }
        trace!("read {}", n);
        // GET /{path}/{id}?tip={host} HTTP/1.1 id をキーに host を保存
        if let Some(idx) = buf[0..n].iter().position(|&x| x == b'\n') {
            let line = String::from_utf8(buf[0..idx].to_vec()).unwrap();
            let replaced_line = line.replace(tip_host, ip_address_from_peer_cast);
            to.write(replaced_line.as_bytes()).await?;
            to.write(&buf[idx..n]).await?;
            continue;
        }
        to.write(&buf[0..n]).await?;
    }
}

pub async fn proxy_for_get_with_tip(
    incoming: TcpStream,
    ip_address_from_peer_cast: String,
    peer_cast_host: &str,
    channel_id: String,
    tip_host: String,
    id_table: Arc<RwLock<HashMap<String, (String, Instant)>>>,
) -> Result<()> {
    let incoming_addr = incoming.peer_addr()?.to_string();
    let incoming_color = ConsoleColor::random_color(&format!("{}_{}", incoming_addr, channel_id));
    let remote_addr = peer_cast_host;
    let remote_color = ConsoleColor::random_color(&format!("{}_{}", remote_addr, channel_id));

    println!(
        "Accept {} (channel_id: {}, tip: {})",
        proxy_message(
            &incoming_addr,
            &incoming_color,
            &incoming.local_addr()?.to_string(),
            remote_addr,
            &remote_color
        ),
        channel_id,
        tip_host,
    );

    let (incoming_read, incoming_write) = incoming.into_split();
    id_table.write().await.insert(
        channel_id.to_uppercase(),
        (tip_host.clone(), Instant::now()),
    );
    let client_socket = TcpStream::connect(peer_cast_host).await?;
    let (client_read, client_write) = client_socket.into_split();
    let client_color = ConsoleColor::random_color(&format!("{}_{}", peer_cast_host, channel_id));

    spawn(async move {
        pipe_for_get_with_tip(
            incoming_read,
            client_write,
            &tip_host,
            &ip_address_from_peer_cast,
        )
        .await
    });
    spawn(async move { pipe(client_read, incoming_write, client_color).await });
    Ok(())
}
