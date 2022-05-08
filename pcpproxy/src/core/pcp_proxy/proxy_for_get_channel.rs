use std::net::IpAddr;
use std::sync::Arc;

use anyhow::Result;
use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::{io::AsyncReadExt, sync::Mutex};

use crate::features::output::ndjson::NDJson;

use super::pipe::big_vec;
use super::pipe::pipe_pcp;
use super::sub_servers::SubServers;

#[derive(Error, Debug)]
#[error("host not found error")]
pub struct HostNotFoundError {}

async fn pipe_http_header(
    incoming: &mut OwnedReadHalf,
    outgoing: &mut OwnedWriteHalf,
    output: NDJson,
) -> Result<()> {
    let mut buf = big_vec(1024 * 1024);
    loop {
        let n = incoming.read(&mut buf).await?;
        outgoing.write_all(&buf[0..n]).await?;
        let text = String::from_utf8_lossy(&buf[0..n]);
        if text.replace('\r', "").ends_with("\n\n") {
            return Ok(());
        }
        output.output_raw(&text);
    }
}

pub async fn proxy_for_get_channel(
    client: TcpStream,
    channel_id: String,
    channel_id_host_pair: &std::sync::Mutex<Vec<(String, String)>>,
) -> Result<()> {
    let local_addr = if let IpAddr::V4(local_addr) = client.local_addr().unwrap().ip() {
        local_addr
    } else {
        unreachable!();
    };
    let client_addr = client.peer_addr().unwrap();

    let (mut client_incoming, mut client_outgoing) = client.into_split();
    log::trace!("read {:?}", channel_id_host_pair.lock().unwrap());
    let server_host = channel_id_host_pair
        .lock()
        .unwrap()
        .iter()
        .find(|(item_channel_id, _item_host)| item_channel_id == &channel_id.to_uppercase())
        .ok_or(HostNotFoundError {})?
        .1
        .to_owned();
    let server = TcpStream::connect(&server_host).await?;
    let (mut server_incoming, mut server_outgoing) = server.into_split();

    let sub_servers = Arc::new(Mutex::new(SubServers::new(local_addr)));
    {
        let sub_servers = sub_servers.clone();
        let client_host = client_addr.to_string();
        let server_host = server_host.clone();
        spawn(async move {
            pipe_http_header(
                &mut client_incoming,
                &mut server_outgoing,
                NDJson::upload(client_host.clone(), server_host.clone()),
            )
            .await?;
            pipe_pcp(
                client_incoming,
                server_outgoing,
                sub_servers.as_ref(),
                NDJson::upload(client_host, server_host),
            )
            .await
        });
    }
    {
        let client_host = client_addr.to_string();
        spawn(async move {
            pipe_http_header(
                &mut server_incoming,
                &mut client_outgoing,
                NDJson::download(client_host.clone(), server_host.clone()),
            )
            .await?;
            pipe_pcp(
                server_incoming,
                client_outgoing,
                sub_servers.as_ref(),
                NDJson::download(client_host, server_host),
            )
            .await
        });
    }
    Ok(())
}
