use std::sync::Arc;

use anyhow::Result;
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
    features::output::ndjson::NDJson,
};

async fn pipe_request_header(
    incoming: &mut BufReader<OwnedReadHalf>,
    outgoing: &mut OwnedWriteHalf,
    host_from_real_server: &str,
    channel_id_host_pair_list: &std::sync::Mutex<Vec<(String, String)>>,
    output: &NDJson,
) -> Result<bool, PipeError> {
    let mut all = String::new();
    let mut channel_id_host_pair = None;
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
        let pattern = r"^GET /(?:pls|stream)/([0-9A-Fa-f]+)\?tip=([^&]+).* HTTP/.+\r?\n$";
        if let Some(capture) = Regex::new(pattern).unwrap().captures(&line) {
            let channel_id = capture[1].to_owned();
            let tip_host = capture[2].to_owned();
            line = line.replace(&tip_host, host_from_real_server);
            channel_id_host_pair = Some((channel_id, tip_host));
        }
        outgoing
            .write_all(line.as_bytes())
            .await
            .map_err(|err| PipeError::ByOutgoing(anyhow::Error::new(err)))?;
        if line.trim_end().is_empty() {
            break;
        }
    }
    output.output_raw(&all);
    if let Some((channel_id, tip_host)) = channel_id_host_pair {
        output.info(&format!(
            "Proxy: Replaced {} with {}",
            tip_host, host_from_real_server
        ));
        output.info(&format!(
            "Proxy: Save {} in {} of the id-tip table.",
            tip_host, channel_id
        ));
        channel_id_host_pair_list
            .lock()
            .unwrap()
            .push((channel_id, tip_host));
    }
    Ok(true)
}

async fn pipe_response_header(
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
    host_from_real_server: &str,
    channel_id_host_pair_list: &std::sync::Mutex<Vec<(String, String)>>,
    output: &NDJson,
) -> Result<(), PipeError> {
    let mut incoming = BufReader::new(incoming);
    loop {
        if !pipe_request_header(
            &mut incoming,
            &mut outgoing,
            host_from_real_server,
            channel_id_host_pair_list,
            output,
        )
        .await?
        {
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
    host_from_real_server: String,
    channel_id_host_pair_list: Arc<std::sync::Mutex<Vec<(String, String)>>>,
) -> Result<()> {
    let client_host = format!("{}", client.peer_addr().unwrap());
    let (client_incoming, client_outgoing) = client.into_split();
    let server = TcpStream::connect(server_host).await?;
    let (server_incoming, server_outgoing) = server.into_split();
    let server_host_string = server_host.into();
    let client_host_clone = client_host.clone();
    spawn(async move {
        let output = NDJson::upload(client_host_clone, server_host_string);
        let result = pipe_http_request(
            client_incoming,
            server_outgoing,
            &host_from_real_server,
            &channel_id_host_pair_list,
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
