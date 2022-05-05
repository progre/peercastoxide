use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use log::*;
use regex::Regex;
use tokio::net::tcp::ReadHalf;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::RwLock;

use crate::core::legacy::console::console_color::ConsoleColor;

use super::pipe::pipe;
use super::proxy_for_get_channel::proxy_for_get_channel;
use super::proxy_for_get_with_tip::proxy_for_get_with_tip;

fn find_channel_id_and_tip_host(line: &str) -> Option<(String, String)> {
    let regex =
        Regex::new(r"/[0-9A-Za-z]+/([0-9A-Fa-f]+)\?tip=([0-9]+\.[0-9]+\.[0-9]+\.[0-9]+:[0-9]+)")
            .unwrap();
    let captures = &regex.captures(line)?;
    Some((captures[1].into(), captures[2].into()))
}

fn find_channel_id(line: &str) -> Option<String> {
    let regex = Regex::new(r"/channel/([0-9A-Fa-f]+)").unwrap();
    let captures = &regex.captures(line)?;
    Some(captures[1].into())
}

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

enum Header {
    GetChannel {
        channel_id: String,
    },
    GetWithTip {
        channel_id: String,
        tip_host: String,
    },
    Pcp,
    Http,
    Unknown,
    Empty,
}

async fn check_header(from: &mut ReadHalf<'_>) -> Result<Header> {
    let mut buf = [0u8; 1024];
    let n = from.peek(&mut buf).await?;
    if n == 0 {
        return Ok(Header::Empty);
    }
    if n > 4 && &buf[0..4] == b"GET " {
        if let Some(idx) = buf[0..n].iter().position(|&x| x == b'\n') {
            let line = String::from_utf8(buf[0..idx].to_vec()).unwrap();
            if let Some(channel_id) = find_channel_id(&line) {
                return Ok(Header::GetChannel { channel_id });
            }
            if let Some((channel_id, tip_host)) = find_channel_id_and_tip_host(&line) {
                return Ok(Header::GetWithTip {
                    channel_id,
                    tip_host,
                });
            }
            return Ok(Header::Http);
        }
    }
    if n > 4 && &buf[0..4] == b"pcp\n" {
        return Ok(Header::Pcp);
    }
    if n > 4 && &buf[0..4] == b"POST" {
        return Ok(Header::Http);
    }
    trace!(
        "Unknown: {:?}, {}",
        &buf[0..4],
        buf[0..4]
            .iter()
            .map(|&x| (x as char).to_string())
            .collect::<Vec<_>>()
            .join("")
    );
    Ok(Header::Unknown)
}

pub async fn server(ip_address_from_peer_cast: &str, peer_cast_host: &str) -> Result<()> {
    let port: u16 = ip_address_from_peer_cast.split(':').collect::<Vec<_>>()[1].parse()?;
    debug!("Listen {}", port);
    let id_table = Arc::new(RwLock::new(HashMap::<String, (String, Instant)>::new()));
    let server = TcpListener::bind(&format!("127.0.0.1:{}", port)).await?;
    loop {
        let (mut incoming_socket, _) = server.accept().await?;
        let (mut server_read, _) = incoming_socket.split();
        let header = check_header(&mut server_read).await?;
        match header {
            Header::GetWithTip {
                channel_id,
                tip_host,
            } => {
                proxy_for_get_with_tip(
                    incoming_socket,
                    ip_address_from_peer_cast.into(),
                    peer_cast_host,
                    channel_id,
                    tip_host,
                    id_table.clone(),
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
                    peer_cast_host,
                );
                proxy(incoming_socket, peer_cast_host).await?;
            }
            Header::Pcp => {
                println!(
                    "Accept {} --> {}(me) --> {} (PCP)",
                    incoming_socket.peer_addr().unwrap(),
                    incoming_socket.local_addr().unwrap(),
                    peer_cast_host,
                );
                proxy(incoming_socket, peer_cast_host).await?;
            }
            Header::Unknown => {
                println!(
                    "Accept {} --> {}(me) --> {} (Unknown packet)",
                    incoming_socket.peer_addr().unwrap(),
                    incoming_socket.local_addr().unwrap(),
                    peer_cast_host,
                );
                proxy(incoming_socket, peer_cast_host).await?;
            }
            Header::Empty => {
                println!(
                    "Accept {} --> {}(me) --> {} (Empty packet)",
                    incoming_socket.peer_addr().unwrap(),
                    incoming_socket.local_addr().unwrap(),
                    peer_cast_host,
                );
            }
        };
    }
}
