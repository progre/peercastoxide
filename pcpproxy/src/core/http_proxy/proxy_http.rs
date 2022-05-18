use anyhow::Result;
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

async fn pipe_header(
    incoming: &mut OwnedReadHalf,
    outgoing: &mut OwnedWriteHalf,
    output: &NDJson,
) -> Result<(), PipeError> {
    let mut br = BufReader::new(incoming);
    let mut all = String::new();
    loop {
        let mut line = String::new();
        br.read_line(&mut line)
            .await
            .map_err(|e| PipeError::ByIncoming(anyhow::anyhow!(e)))?;
        outgoing
            .write_all(line.as_bytes())
            .await
            .map_err(|e| PipeError::ByOutgoing(anyhow::anyhow!(e)))?;
        all += &line;
        if line.trim_end().is_empty() {
            break;
        }
    }
    output.output_raw(&all);
    Ok(())
}

async fn pipe_http(
    mut incoming: OwnedReadHalf,
    mut outgoing: OwnedWriteHalf,
    output: &NDJson,
) -> Result<(), PipeError> {
    pipe_header(&mut incoming, &mut outgoing, output).await?;
    pipe_raw(incoming, outgoing, output).await
}

pub async fn proxy_http(client: TcpStream, server_host: &str) -> Result<()> {
    let client_host = format!("{}", client.peer_addr().unwrap());
    let (client_incoming, client_outgoing) = client.into_split();
    let server = TcpStream::connect(server_host).await?;
    let (server_incoming, server_outgoing) = server.into_split();
    let server_host_string = server_host.into();
    let client_host_clone = client_host.clone();
    spawn(async move {
        let output = NDJson::upload(client_host_clone, server_host_string);
        let result = pipe_http(client_incoming, server_outgoing, &output).await;
        disconnect_conn_of_upload(result, output).unwrap();
    });
    let server_host_string = server_host.into();
    spawn(async move {
        let output = NDJson::download(client_host, server_host_string);
        let result = pipe_http(server_incoming, client_outgoing, &output).await;
        disconnect_conn_of_download(result, output).unwrap();
    });
    Ok(())
}
