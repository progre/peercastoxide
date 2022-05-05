use anyhow::Result;
use derive_new::new;
use log::*;
use std::{collections::HashMap, net::Ipv4Addr};
use tokio::prelude::*;
use tokio::{net::TcpListener, spawn};

#[derive(new)]
pub struct SubServers {
    local_addr: Ipv4Addr,
    #[new(default)]
    port_map: HashMap<u16, (Ipv4Addr, u16)>,
}

async fn accept(server: TcpListener) -> Result<()> {
    loop {
        let (mut incoming_socket, _) = server.accept().await?;
        let (mut server_read, _) = incoming_socket.split();

        let mut buf = [0u8; 4];
        server_read.read(&mut buf).await?;
        trace!(
            "sub: {}",
            buf.iter()
                .map(|&x| (x as char).to_string())
                .collect::<Vec<_>>()
                .join("")
        );
    }
}

impl SubServers {
    pub async fn start_server(
        &mut self,
        original_addr: Ipv4Addr,
        original_port: u16,
    ) -> Result<(Ipv4Addr, u16)> {
        let server = TcpListener::bind((self.local_addr.clone(), 0u16)).await?;
        let port = server.local_addr().unwrap().port();
        self.port_map.insert(port, (original_addr, original_port));
        spawn(async move {
            accept(server).await.unwrap();
        });
        trace!("sub server started {} {}", self.local_addr, port);
        Ok((self.local_addr, port))
    }
}