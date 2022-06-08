use std::net::{IpAddr, SocketAddr};
use std::{num::NonZeroU16, time::Duration};

use regex::Regex;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::{net::TcpListener, spawn, time::timeout};

use crate::core::http_proxy::proxy_http::pipe_request_header;
use crate::core::http_proxy::proxy_http::pipe_response_header;
use crate::core::pcp_proxy::pipe::pipe_pcp;
use crate::core::utils::disconnect_conn_of_download;
use crate::core::utils::disconnect_conn_of_upload;
use crate::features::output::ndjson::NDJson;

async fn on_connect(
    client: TcpStream,
    real_server_port: NonZeroU16,
    ip_addr_from_real_server: IpAddr,
    listen_port: NonZeroU16,
    tip_host: SocketAddr,
) -> anyhow::Result<()> {
    let client_addr = client.peer_addr().unwrap();

    let (client_incoming, mut client_outgoing) = client.into_split();
    let server = TcpStream::connect(&tip_host).await?;
    let (server_incoming, mut server_outgoing) = server.into_split();

    {
        let client_host = client_addr.to_string();
        spawn(async move {
            let output = NDJson::upload(client_host, tip_host.to_string());
            let result = async {
                let replacement_pair = std::sync::Mutex::new(None);
                let mut client_incoming = BufReader::new(client_incoming);
                pipe_request_header(
                    &mut client_incoming,
                    &mut server_outgoing,
                    |mut line| async {
                        let pattern = r"^Host: ?([^\r\n]+)\r?\n$";
                        if let Some(capture) = Regex::new(pattern).unwrap().captures(&line) {
                            let my_host = capture[1].to_owned();
                            line = line.replace(&my_host, &tip_host.to_string());
                            *replacement_pair.lock().unwrap() = Some((my_host, tip_host));
                        }
                        line
                    },
                    &output,
                )
                .await?;
                if let Some((from, to)) = replacement_pair.lock().unwrap().as_ref() {
                    output.info(&format!("Proxy: Replaced {} with {}", from, to));
                }
                pipe_pcp(
                    client_incoming,
                    server_outgoing,
                    real_server_port,
                    ip_addr_from_real_server,
                    listen_port,
                    &output,
                )
                .await
            }
            .await;
            disconnect_conn_of_upload(result, output).unwrap();
        });
    }
    {
        let client_host = client_addr.to_string();
        spawn(async move {
            let output = NDJson::download(client_host.clone(), tip_host.to_string());
            let result = async {
                let mut server_incoming = BufReader::new(server_incoming);
                pipe_response_header(
                    &mut server_incoming,
                    &mut client_outgoing,
                    |line| async { line },
                    &output,
                )
                .await?;
                pipe_pcp(
                    server_incoming,
                    client_outgoing,
                    real_server_port,
                    ip_addr_from_real_server,
                    listen_port,
                    &output,
                )
                .await
            }
            .await;
            disconnect_conn_of_download(result, output).unwrap();
        });
    }
    Ok(())
}

fn spawn_listener(
    server: TcpListener,
    real_server_port: NonZeroU16,
    ip_addr_from_real_server: IpAddr,
    listen_port: NonZeroU16,
    tip_host: SocketAddr,
) {
    spawn(async move {
        let result = match timeout(Duration::from_secs(10), server.accept()).await {
            Ok(ok) => ok,
            Err(_) => return,
        };
        let (client, _) = result.unwrap();
        on_connect(
            client,
            real_server_port,
            ip_addr_from_real_server,
            listen_port,
            tip_host,
        )
        .await
        .unwrap();
    });
}

pub async fn listen_for(
    real_server_port: NonZeroU16,
    ip_addr_from_real_server: IpAddr,
    listen_port: NonZeroU16,
    tip_host: SocketAddr,
) -> NonZeroU16 {
    let server = TcpListener::bind(SocketAddr::new(ip_addr_from_real_server, 0))
        .await
        .unwrap();
    let port = server.local_addr().unwrap().port().try_into().unwrap();
    spawn_listener(
        server,
        real_server_port,
        ip_addr_from_real_server,
        listen_port,
        tip_host,
    );
    port
}
