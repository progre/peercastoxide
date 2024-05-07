use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
    time::Duration,
};

use anyhow::{anyhow, Result};
use peercastoxide_lib::pcp::atom::{
    values::Id, well_known_protocols::handshake, AtomStreamReader, AtomStreamWriter,
};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    select, spawn,
    task::JoinHandle,
    time::timeout,
};
use tracing::error;

const AGENT_NAME: &str = concat!("PeerCastOxide/", env!("CARGO_PKG_VERSION"));

async fn process(stream: TcpStream) -> Result<()> {
    let session_id = Id(rand::random());
    let peer_addr = stream.peer_addr()?;
    let (reader, writer) = stream.into_split();
    let mut reader = AtomStreamReader::new(reader);
    let mut writer = AtomStreamWriter::new(writer);

    timeout(
        Duration::from_secs(15),
        handshake(
            &mut reader,
            &mut writer,
            &session_id,
            peer_addr.ip(),
            AGENT_NAME,
            Duration::from_secs(5),
        ),
    )
    .await??;

    loop {
        let atom = reader
            .read_unknown_atom()
            .await
            .map_err(|e| anyhow!("failed: {:?}", e))?;
        // let bcst: Bcst = atom::deserializer_from_atom::from_atom(atom)?;
        tracing::trace!("{:#}", atom);
    }
}

async fn accept_connenctions_loop(addr: impl ToSocketAddrs) -> Result<()> {
    loop {
        let listener = TcpListener::bind(&addr).await?;
        let (socket, _) = listener.accept().await?;
        tracing::trace!("accept: {}", socket.peer_addr()?);
        spawn(async move {
            if let Err(e) = process(socket).await {
                error!("{:?}", e);
            }
        });
    }
}

pub async fn listen(port: u16) -> anyhow::Result<()> {
    tracing::trace!("listen");
    let socket = SocketAddr::new(IpAddr::from_str("0.0.0.0").unwrap(), port);
    let v4: JoinHandle<Result<()>> = spawn(accept_connenctions_loop(socket));
    let socket = SocketAddr::new(IpAddr::from_str("::").unwrap(), port);
    let v6: JoinHandle<Result<()>> = spawn(accept_connenctions_loop(socket));
    select! {
        result = v4 => {
            result??;
        }
        result = v6 => {
            result??;
        }
    }
    Ok(())
}
