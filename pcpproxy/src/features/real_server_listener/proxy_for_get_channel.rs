use std::net::IpAddr;
use std::sync::Arc;

use thiserror::Error;
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::{io::AsyncReadExt, sync::Mutex};

use crate::core::pcp_proxy::pipe::big_vec;
use crate::core::pcp_proxy::pipe::pipe_pcp;
use crate::core::pcp_proxy::sub_servers::SubServers;
use crate::core::utils::disconnect_conn_of_download;
use crate::core::utils::disconnect_conn_of_upload;
use crate::core::utils::PipeError;
use crate::features::output::ndjson::NDJson;

#[derive(Error, Debug)]
#[error("host not found error")]
pub struct HostNotFoundError {}

async fn pipe_http_header(
    incoming: &mut OwnedReadHalf,
    outgoing: &mut OwnedWriteHalf,
    output: &NDJson,
) -> Result<(), PipeError> {
    let mut buf = big_vec(1024 * 1024);
    loop {
        let n = incoming
            .read(&mut buf)
            .await
            .map_err(|err| PipeError::ByIncoming(anyhow::Error::new(err)))?;
        outgoing
            .write_all(&buf[0..n])
            .await
            .map_err(|err| PipeError::ByOutgoing(anyhow::Error::new(err)))?;
        let text = String::from_utf8_lossy(&buf[0..n]);
        if text.replace('\r', "").ends_with("\n\n") {
            return Ok(());
        }
        output.output_raw(&text);
    }
}

pub async fn proxy_for_get_channel(client: TcpStream, tip_host: String) -> anyhow::Result<()> {
    let local_addr = if let IpAddr::V4(local_addr) = client.local_addr().unwrap().ip() {
        local_addr
    } else {
        unreachable!();
    };
    let client_addr = client.peer_addr().unwrap();

    let (mut client_incoming, mut client_outgoing) = client.into_split();
    let server = TcpStream::connect(&tip_host).await?;
    let (mut server_incoming, mut server_outgoing) = server.into_split();

    let sub_servers = Arc::new(Mutex::new(SubServers::new(local_addr)));
    {
        let sub_servers = sub_servers.clone();
        let client_host = client_addr.to_string();
        let tip_host = tip_host.clone();
        spawn(async move {
            let output = NDJson::upload(client_host, tip_host);
            let result = async {
                pipe_http_header(&mut client_incoming, &mut server_outgoing, &output).await?;
                pipe_pcp(client_incoming, server_outgoing, &sub_servers, &output).await
            }
            .await;
            disconnect_conn_of_upload(result, output).unwrap();
        });
    }
    {
        let client_host = client_addr.to_string();
        spawn(async move {
            let output = NDJson::download(client_host.clone(), tip_host);
            let result = async {
                pipe_http_header(&mut server_incoming, &mut client_outgoing, &output).await?;
                pipe_pcp(server_incoming, client_outgoing, &sub_servers, &output).await
            }
            .await;
            disconnect_conn_of_download(result, output).unwrap();
        });
    }
    Ok(())
}
