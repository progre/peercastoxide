use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;
use std::net::SocketAddr;
use std::num::NonZeroU16;

use anyhow::Result;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::spawn;

use crate::core::pcp_proxy::header::check_header;
use crate::core::pcp_proxy::header::Header;
use crate::features::output::ndjson::NDJson;

use super::http_proxy::proxy_http::proxy_http;
use super::pcp_proxy::pipe::pipe_pcp;
use super::utils::disconnect_conn_of_download;
use super::utils::disconnect_conn_of_upload;
use super::utils::pipe_raw;

async fn proxy_pcp(
    client: TcpStream,
    server_host: &str,
    ip_addr_from_real_server: IpAddr,
    listen_port: NonZeroU16,
) -> Result<()> {
    let client_host = format!("{}", client.peer_addr().unwrap());
    let (client_incoming, client_outgoing) = client.into_split();
    let server = TcpStream::connect(server_host).await?;
    let (server_incoming, server_outgoing) = server.into_split();
    let server_host_string = server_host.into();
    let client_host_clone = client_host.clone();
    let real_server_port = server_host.split(':').nth(1).unwrap().parse().unwrap();
    spawn(async move {
        let output = NDJson::upload(client_host_clone, server_host_string);
        let result = pipe_pcp(
            client_incoming,
            server_outgoing,
            real_server_port,
            ip_addr_from_real_server,
            listen_port,
            &output,
        )
        .await;
        disconnect_conn_of_upload(result, output).unwrap();
    });
    let server_host_string = server_host.into();
    spawn(async move {
        let output = NDJson::download(client_host, server_host_string);
        let result = pipe_pcp(
            server_incoming,
            client_outgoing,
            real_server_port,
            ip_addr_from_real_server,
            listen_port,
            &output,
        )
        .await;
        disconnect_conn_of_download(result, output).unwrap()
    });
    Ok(())
}

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
        disconnect_conn_of_upload(result, output).unwrap();
    });
    let server_host_string = server_host.into();
    spawn(async move {
        let output = NDJson::download(client_host, server_host_string);
        let result = pipe_raw(server_incoming, client_outgoing, &output).await;
        disconnect_conn_of_download(result, output).unwrap()
    });
    Ok(())
}

async fn on_connect(
    mut client: TcpStream,
    ip_addr_from_real_server: IpAddr,
    port: NonZeroU16,
    real_server_host: &str,
) -> Result<()> {
    let (mut client_incoming, _) = client.split();
    let header = check_header(&mut client_incoming).await?;
    match header {
        Header::Http => {
            proxy_http(client, real_server_host, ip_addr_from_real_server, port).await?;
        }
        Header::Pcp => {
            proxy_pcp(client, real_server_host, ip_addr_from_real_server, port).await?;
        }
        Header::Unknown => {
            proxy_raw(client, real_server_host).await?;
        }
        Header::Empty => {}
    };
    Ok(())
}

pub async fn listen(
    listen_port: NonZeroU16,
    ip_addr_from_real_server: IpAddr,
    real_server_host: &str,
) {
    let unspecified_addr: IpAddr = match ip_addr_from_real_server {
        IpAddr::V4(_) => Ipv4Addr::UNSPECIFIED.into(),
        IpAddr::V6(_) => Ipv6Addr::UNSPECIFIED.into(),
    };

    let server = TcpListener::bind(SocketAddr::new(unspecified_addr, listen_port.get()))
        .await
        .unwrap();
    loop {
        let (incoming_socket, _) = server.accept().await.unwrap();
        let real_server_host = real_server_host.to_owned();
        spawn(async move {
            on_connect(
                incoming_socket,
                ip_addr_from_real_server,
                listen_port,
                &real_server_host,
            )
            .await
            .unwrap();
        });
    }
}
