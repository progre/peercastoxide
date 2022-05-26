use std::{net::Ipv4Addr, num::NonZeroU16};

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
    core::utils::{disconnect_conn_of_download, disconnect_conn_of_upload, pipe_raw, PipeError},
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

pub async fn pipe_response_header(
    incoming: &mut BufReader<OwnedReadHalf>,
    outgoing: &mut OwnedWriteHalf,
    output: &NDJson,
) -> Result<bool, PipeError> {
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
        if line.trim_end().is_empty() {
            break;
        }
    }
    output.output_raw(&all);
    Ok(true)
}

async fn pipe_http_request(
    incoming: OwnedReadHalf,
    mut outgoing: OwnedWriteHalf,
    real_server_ipv4_port: NonZeroU16,
    ipv4_addr_from_real_server: Ipv4Addr,
    ipv4_port: NonZeroU16,
    output: &NDJson,
) -> Result<(), PipeError> {
    let mut incoming = BufReader::new(incoming);
    loop {
        let replacement_pair = std::sync::Mutex::new(None);
        let remain = pipe_request_header(
            &mut incoming,
            &mut outgoing,
            |mut line| async {
                let pattern = r"^GET /(?:pls|stream)/(?:[0-9A-Fa-f]+)\?tip=([^&]+).* HTTP/.+\r?\n$";
                if let Some(capture) = Regex::new(pattern).unwrap().captures(&line) {
                    let tip_host = capture[1].to_owned();
                    let port = listen_for(
                        real_server_ipv4_port,
                        ipv4_addr_from_real_server,
                        ipv4_port,
                        tip_host.clone(),
                    )
                    .await;
                    let replace_with = format!("{}:{}", ipv4_addr_from_real_server, port);
                    line = line.replace(&tip_host, &replace_with);
                    *replacement_pair.lock().unwrap() = Some((tip_host, replace_with));
                }
                line
            },
            output,
        )
        .await?;
        if let Some((from, to)) = replacement_pair.lock().unwrap().as_ref() {
            output.info(&format!("Proxy: Replaced {} with {}", from, to));
        }
        if !remain {
            return Ok(());
        }
    }
}

async fn pipe_http_response(
    incoming: OwnedReadHalf,
    mut outgoing: OwnedWriteHalf,
    output: &NDJson,
) -> Result<(), PipeError> {
    let mut incoming = BufReader::new(incoming);
    if !pipe_response_header(&mut incoming, &mut outgoing, output).await? {
        return Ok(());
    }
    pipe_raw(incoming, outgoing, output).await
}

pub async fn proxy_http(
    client: TcpStream,
    server_host: &str,
    ipv4_addr_from_real_server: Ipv4Addr,
    ipv4_port: NonZeroU16,
) -> Result<()> {
    let client_host = format!("{}", client.peer_addr().unwrap());
    let (client_incoming, client_outgoing) = client.into_split();
    let server = TcpStream::connect(server_host).await?;
    let (server_incoming, server_outgoing) = server.into_split();
    let server_host_string = server_host.into();
    let client_host_clone = client_host.clone();
    let real_server_ipv4_port = server_host.split(':').nth(1).unwrap().parse().unwrap();
    spawn(async move {
        let output = NDJson::upload(client_host_clone, server_host_string);
        let result = pipe_http_request(
            client_incoming,
            server_outgoing,
            real_server_ipv4_port,
            ipv4_addr_from_real_server,
            ipv4_port,
            &output,
        )
        .await;
        disconnect_conn_of_upload(result, output).unwrap();
    });
    let server_host_string = server_host.into();
    spawn(async move {
        let output = NDJson::download(client_host, server_host_string);
        let result = pipe_http_response(server_incoming, client_outgoing, &output).await;
        disconnect_conn_of_download(result, output).unwrap();
    });
    Ok(())
}
