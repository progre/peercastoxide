use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use log::*;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::RwLock;

use crate::core::pcp_proxy::header::check_header;
use crate::core::pcp_proxy::header::Header;
use crate::core::pcp_proxy::pipe::pipe;
use crate::core::pcp_proxy::proxy_for_get_channel::proxy_for_get_channel;
use crate::core::pcp_proxy::proxy_for_get_with_tip::proxy_for_get_with_tip;
use crate::features::console::console_color::ConsoleColor;

async fn proxy(incoming: TcpStream, host: &str) -> Result<()> {
    let incoming_color = ConsoleColor::random_color(&format!("{}", incoming.peer_addr().unwrap()));
    let (incoming_read, incoming_write) = incoming.into_split();
    let client_socket = TcpStream::connect(host).await?;
    let (client_read, client_write) = client_socket.into_split();
    let client_color = ConsoleColor::random_color(host);
    spawn(async move { pipe(incoming_read, client_write, incoming_color).await });
    spawn(async move { pipe(client_read, incoming_write, client_color).await });
    Ok(())
}

async fn on_connect(
    mut incoming_socket: TcpStream,
    id_table: Arc<RwLock<HashMap<String, (String, Instant)>>>,
    my_global_host: &str,
    real_server_host: &str,
) -> Result<()> {
    let (mut server_read, _) = incoming_socket.split();
    let header = check_header(&mut server_read).await?;
    match header {
        Header::GetWithTip {
            channel_id,
            tip_host,
        } => {
            proxy_for_get_with_tip(
                incoming_socket,
                my_global_host.into(),
                real_server_host,
                channel_id,
                tip_host,
                id_table,
            )
            .await?;
        }
        Header::GetChannel { channel_id } => {
            proxy_for_get_channel(incoming_socket, channel_id, id_table.clone()).await?
        }
        Header::Http => {
            println!(
                "Accept {} --> {}(me) --> {} (Standard HTTP GET Request)",
                incoming_socket.peer_addr().unwrap(),
                incoming_socket.local_addr().unwrap(),
                real_server_host,
            );
            proxy(incoming_socket, real_server_host).await?;
        }
        Header::Pcp => {
            println!(
                "Accept {} --> {}(me) --> {} (PCP)",
                incoming_socket.peer_addr().unwrap(),
                incoming_socket.local_addr().unwrap(),
                real_server_host,
            );
            proxy(incoming_socket, real_server_host).await?;
        }
        Header::Unknown => {
            println!(
                "Accept {} --> {}(me) --> {} (Unknown packet)",
                incoming_socket.peer_addr().unwrap(),
                incoming_socket.local_addr().unwrap(),
                real_server_host,
            );
            proxy(incoming_socket, real_server_host).await?;
        }
        Header::Empty => {
            println!(
                "Accept {} --> {}(me) --> {} (Empty packet)",
                incoming_socket.peer_addr().unwrap(),
                incoming_socket.local_addr().unwrap(),
                real_server_host,
            );
        }
    };
    Ok(())
}

pub async fn listen(ip_address_from_peer_cast: &str, peer_cast_host: &str) -> Result<()> {
    let port: u16 = ip_address_from_peer_cast.split(':').collect::<Vec<_>>()[1].parse()?;
    debug!("Listen {}", port);
    let id_table = Arc::new(RwLock::new(HashMap::<String, (String, Instant)>::new()));
    let server = TcpListener::bind(&format!("127.0.0.1:{}", port)).await?;
    loop {
        let (incoming_socket, _) = server.accept().await?;
        on_connect(
            incoming_socket,
            id_table.clone(),
            ip_address_from_peer_cast,
            peer_cast_host,
        )
        .await?;
    }
}
