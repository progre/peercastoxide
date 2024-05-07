use std::{
    net::{IpAddr, SocketAddr},
    num::NonZeroU16,
    str::FromStr,
};

use anyhow::Result;
use futures::Future;
use regex::Regex;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    spawn,
};

use crate::{
    core::{
        pcp_proxy::pipe::pipe_pcp,
        utils::{disconnect_conn_of_download, disconnect_conn_of_upload, pipe_raw, PipeError},
    },
    features::{output::ndjson::NDJson, real_server_listener::listen_for::listen_for},
};

pub async fn pipe_request_header<T>(
    incoming: &mut BufReader<OwnedReadHalf>,
    outgoing: &mut OwnedWriteHalf,
    on_line: impl Fn(String) -> T,
    output: &NDJson,
) -> Result<bool, PipeError>
where
    T: Future<Output = String>,
{
    let mut all = String::new();

    loop {
        let mut line = String::new();
        let result = incoming
            .read_line(&mut line)
            .await
            .map_err(|err| PipeError::ByIncoming(anyhow::Error::new(err)))?;
        if result == 0 {
            return Ok(false);
        }
        all += &line;
        line = on_line(line).await;
        outgoing
            .write_all(line.as_bytes())
            .await
            .map_err(|err| PipeError::ByOutgoing(anyhow::Error::new(err)))?;
        if line.trim_end().is_empty() {
            break;
        }
    }
    output.output_raw(&all);
    Ok(true)
}

pub async fn pipe_response_header<T>(
    incoming: &mut BufReader<OwnedReadHalf>,
    outgoing: &mut OwnedWriteHalf,
    on_line: impl Fn(String) -> T,
    output: &NDJson,
) -> Result<bool, PipeError>
where
    T: Future<Output = String>,
{
    let mut all = String::new();
    loop {
        let mut line = String::new();
        let result = incoming
            .read_line(&mut line)
            .await
            .map_err(|err| PipeError::ByIncoming(anyhow::Error::new(err)))?;
        if result == 0 {
            return Ok(false);
        }
        outgoing
            .write_all(line.as_bytes())
            .await
            .map_err(|err| PipeError::ByOutgoing(anyhow::Error::new(err)))?;
        all += &line;
        line = on_line(line).await;
        if line.trim_end().is_empty() {
            break;
        }
    }
    output.output_raw(&all);
    Ok(true)
}

async fn pipe_http_request(
    incoming: &mut BufReader<OwnedReadHalf>,
    outgoing: &mut OwnedWriteHalf,
    real_server_port: NonZeroU16,
    ip_addr_from_real_server: IpAddr,
    port: NonZeroU16,
    output: &NDJson,
) -> Result<bool, PipeError> {
    loop {
        let replacement_pair = std::sync::Mutex::new(None);
        let pcp = std::sync::Mutex::new(false);
        let remain = pipe_request_header(
            incoming,
            outgoing,
            |mut line| async {
                let pattern = r"^GET /(?:pls|stream)/(?:[0-9A-Fa-f]+)\?tip=([^&]+).* HTTP/.+\r?\n$";
                if let Some(capture) = Regex::new(pattern).unwrap().captures(&line) {
                    let tip_host = capture[1].to_owned();
                    let port = listen_for(
                        real_server_port,
                        ip_addr_from_real_server,
                        port,
                        SocketAddr::from_str(&tip_host).unwrap(),
                    )
                    .await;
                    let replace_with =
                        SocketAddr::new(ip_addr_from_real_server, port.get()).to_string();
                    line = line.replace(&tip_host, &replace_with);
                    *replacement_pair.lock().unwrap() = Some((tip_host, replace_with));
                }
                if line.starts_with("x-peercast-pcp:") {
                    *pcp.lock().unwrap() = true;
                }
                line
            },
            output,
        )
        .await?;
        if let Some((from, to)) = replacement_pair.lock().unwrap().as_ref() {
            output.info(&format!("Proxy: Replaced {} with {}", from, to));
        }
        if *pcp.lock().unwrap() {
            return Ok(true);
        }
        if !remain {
            return Ok(false);
        }
    }
}

fn is_pcp(line: &str) -> bool {
    let mut header = line.split(':');
    header.next().unwrap().trim() == "Content-Type"
        && header.next().unwrap().trim() == "application/x-peercast-pcp"
}

async fn pipe_http_response(
    incoming: &mut BufReader<OwnedReadHalf>,
    outgoing: &mut OwnedWriteHalf,
    output: &NDJson,
) -> Result<bool, PipeError> {
    let pcp = std::sync::Mutex::new(false);
    if !pipe_response_header(
        incoming,
        outgoing,
        |line| async {
            if is_pcp(&line) {
                *pcp.lock().unwrap() = true;
            }
            line
        },
        output,
    )
    .await?
    {
        return Ok(false);
    }
    if *pcp.lock().unwrap() {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn proxy_http(
    client: TcpStream,
    real_server_host: &str,
    ip_addr_from_real_server: IpAddr,
    listen_port: NonZeroU16,
) -> Result<()> {
    let client_host = format!("{}", client.peer_addr().unwrap());
    let (client_incoming, mut client_outgoing) = client.into_split();
    let server = TcpStream::connect(real_server_host).await?;
    let (server_incoming, mut server_outgoing) = server.into_split();
    let server_host_string = real_server_host.into();
    let client_host_clone = client_host.clone();
    let real_server_port = real_server_host.split(':').nth(1).unwrap().parse().unwrap();
    spawn(async move {
        let mut client_incoming = BufReader::new(client_incoming);
        let output = NDJson::upload(client_host_clone, server_host_string);
        let result = pipe_http_request(
            &mut client_incoming,
            &mut server_outgoing,
            real_server_port,
            ip_addr_from_real_server,
            listen_port,
            &output,
        )
        .await;
        let result = if let Ok(true) = result {
            pipe_pcp(
                client_incoming,
                server_outgoing,
                real_server_port,
                ip_addr_from_real_server,
                listen_port,
                &output,
            )
            .await
        } else {
            result.map(|_| ())
        };
        disconnect_conn_of_upload(result, output).unwrap();
    });
    let server_host_string = real_server_host.into();
    spawn(async move {
        let mut server_incoming = BufReader::new(server_incoming);
        let output = NDJson::download(client_host, server_host_string);
        let result = pipe_http_response(&mut server_incoming, &mut client_outgoing, &output).await;
        let result = if let Ok(true) = result {
            pipe_pcp(
                server_incoming,
                client_outgoing,
                real_server_port,
                ip_addr_from_real_server,
                listen_port,
                &output,
            )
            .await
        } else if let Ok(false) = result {
            pipe_raw(server_incoming, client_outgoing, &output).await
        } else {
            result.map(|_| ())
        };
        disconnect_conn_of_download(result, output).unwrap();
    });
    Ok(())
}
