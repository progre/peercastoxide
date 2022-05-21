use std::net::IpAddr;
use std::sync::Arc;
use std::{num::NonZeroU16, time::Duration};

use regex::Regex;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::{net::TcpListener, spawn, time::timeout};

use crate::core::http_proxy::proxy_http::pipe_request_header;
use crate::core::http_proxy::proxy_http::pipe_response_header;
use crate::core::pcp_proxy::pipe::pipe_pcp;
use crate::core::pcp_proxy::sub_servers::SubServers;
use crate::core::utils::disconnect_conn_of_download;
use crate::core::utils::disconnect_conn_of_upload;
use crate::features::output::ndjson::NDJson;

async fn on_connect(client: TcpStream, tip_host: String) -> anyhow::Result<()> {
    let local_addr = if let IpAddr::V4(local_addr) = client.local_addr().unwrap().ip() {
        local_addr
    } else {
        unreachable!();
    };
    let client_addr = client.peer_addr().unwrap();

    let (client_incoming, mut client_outgoing) = client.into_split();
    let server = TcpStream::connect(&tip_host).await?;
    let (server_incoming, mut server_outgoing) = server.into_split();

    let sub_servers = Arc::new(Mutex::new(SubServers::new(local_addr)));
    {
        let sub_servers = sub_servers.clone();
        let client_host = client_addr.to_string();
        let tip_host = tip_host.clone();
        spawn(async move {
            let output = NDJson::upload(client_host, tip_host.clone());
            let result = async {
                let replacement_pair = std::sync::Mutex::new(None);
                let mut client_incoming = BufReader::new(client_incoming);
                pipe_request_header(
                    &mut client_incoming,
                    &mut server_outgoing,
                    |mut line| {
                        let tip_host = tip_host.clone();
                        async {
                            let pattern = r"^Host: ?([^\r\n]+)\r?\n$";
                            if let Some(capture) = Regex::new(pattern).unwrap().captures(&line) {
                                let my_host = capture[1].to_owned();
                                line = line.replace(&my_host, &tip_host);
                                *replacement_pair.lock().unwrap() = Some((my_host, tip_host));
                            }
                            line
                        }
                    },
                    &output,
                )
                .await?;
                if let Some((from, to)) = replacement_pair.lock().unwrap().as_ref() {
                    output.info(&format!("Proxy: Replaced {} with {}", from, to));
                }
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
                let mut server_incoming = BufReader::new(server_incoming);
                pipe_response_header(&mut server_incoming, &mut client_outgoing, &output).await?;
                pipe_pcp(server_incoming, client_outgoing, &sub_servers, &output).await
            }
            .await;
            disconnect_conn_of_download(result, output).unwrap();
        });
    }
    Ok(())
}

pub async fn listen_for(hostname_from_real_server: &str, tip_host: String) -> NonZeroU16 {
    let server = TcpListener::bind(&format!("{}:0", hostname_from_real_server))
        .await
        .unwrap();
    let port = server.local_addr().unwrap().port().try_into().unwrap();
    spawn(async move {
        let (client, _) = timeout(Duration::from_secs(10), server.accept())
            .await
            .unwrap()
            .unwrap();
        on_connect(client, tip_host).await.unwrap();
    });
    port
}
