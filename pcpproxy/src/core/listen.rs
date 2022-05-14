use std::sync::Arc;

use anyhow::Result;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::spawn;

use crate::core::pcp_proxy::header::check_header;
use crate::core::pcp_proxy::header::Header;
use crate::core::pcp_proxy::pipe::pipe_raw;
use crate::core::pcp_proxy::proxy_for_get_channel::proxy_for_get_channel;
use crate::features::output::ndjson::NDJson;

use super::http_proxy::proxy_for_get_with_tip::proxy_for_get_with_tip;
use super::utils::disconnect_conn_of_download;
use super::utils::disconnect_conn_of_upload;

async fn proxy_raw(client: TcpStream, server_host: &str) -> Result<()> {
    let client_host = format!("{}", client.peer_addr().unwrap());
    let (client_incoming, client_outgoing) = client.into_split();
    let server = TcpStream::connect(server_host).await?;
    let (server_incoming, server_outgoing) = server.into_split();
    let server_host_string = server_host.into();
    let client_host_clone = client_host.clone();
    spawn(async move {
        let output = NDJson::upload(client_host_clone, server_host_string);
        let result = pipe_raw(client_incoming, server_outgoing, &output).await;
        disconnect_conn_of_upload(result, output)
    });
    let server_host_string = server_host.into();
    spawn(async move {
        let output = NDJson::download(client_host, server_host_string);
        let result = pipe_raw(server_incoming, client_outgoing, &output).await;
        disconnect_conn_of_download(result, output)
    });
    Ok(())
}

async fn on_connect(
    mut client: TcpStream,
    channel_id_host_pair: &std::sync::Mutex<Vec<(String, String)>>,
    host_from_real_server: &str,
    real_server_host: &str,
) -> Result<()> {
    let (mut client_incoming, _) = client.split();
    let header = check_header(&mut client_incoming).await?;
    match header {
        // ローカルのブラウザーを起点にしてリモートへ pcp over http で情報を取得する
        Header::GetWithTip {
            channel_id,
            tip_host,
        } => {
            proxy_for_get_with_tip(
                client,
                host_from_real_server.into(),
                real_server_host,
                channel_id,
                tip_host,
                channel_id_host_pair,
            )
            .await?;
        }
        // リモートからローカルへの pcp over http 通信
        Header::GetChannel { channel_id } => {
            proxy_for_get_channel(client, channel_id, channel_id_host_pair).await?
        }
        Header::Http => {
            proxy_raw(client, real_server_host).await?;
        }
        Header::Pcp => {
            proxy_raw(client, real_server_host).await?;
        }
        Header::Unknown => {
            proxy_raw(client, real_server_host).await?;
        }
        Header::Empty => {}
    };
    Ok(())
}

pub async fn listen(host_from_real_server: &str, real_server_host: &str) -> Result<()> {
    let port: u16 = host_from_real_server.split(':').collect::<Vec<_>>()[1].parse()?;
    let channel_id_host_pair = Arc::new(std::sync::Mutex::new(Vec::<(String, String)>::new()));
    let server = TcpListener::bind(&format!("0.0.0.0:{}", port)).await?;
    loop {
        let (incoming_socket, _) = server.accept().await?;
        let channel_id_host_pair = channel_id_host_pair.clone();
        let host_from_real_server = host_from_real_server.to_owned();
        let real_server_host = real_server_host.to_owned();
        spawn(async move {
            on_connect(
                incoming_socket,
                &channel_id_host_pair,
                &host_from_real_server,
                &real_server_host,
            )
            .await
        });
    }
}