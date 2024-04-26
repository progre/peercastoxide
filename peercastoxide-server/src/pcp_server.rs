use std::time::Duration;

use anyhow::{anyhow, Result};
use peercastoxide_lib::pcp::{
    atom::well_known_protocols::handshake,
    atom_stream::{AtomStreamReader, AtomStreamWriter},
};
use tokio::{
    net::{TcpListener, TcpStream},
    select, spawn,
    task::JoinHandle,
    time::timeout,
};
use tracing::error;

const AGENT_NAME: &str = concat!("PeerCastOxide/", env!("CARGO_PKG_VERSION"));

async fn process(socket: TcpStream) -> Result<()> {
    let session_id: [u8; 16] = rand::random();
    let peer_addr = socket.peer_addr()?;
    let (read_half, write_half) = socket.into_split();
    let mut read_stream = AtomStreamReader::new(read_half);
    let mut write_stream = AtomStreamWriter::new(write_half);
    let timeout_future = timeout(
        Duration::from_secs(30),
        handshake(
            AGENT_NAME,
            peer_addr.ip(),
            &mut read_stream,
            &mut write_stream,
            &session_id,
            Duration::from_secs(15),
        ),
    );
    timeout_future.await??;

    loop {
        let atom = read_stream
            .read_atom()
            .await
            .map_err(|e| anyhow!("failed: {:?}", e))?
            .ok_or_else(|| anyhow!("failed: end of stream"))?;
        tracing::trace!("{:#}", atom);
    }
}

pub async fn listen() -> anyhow::Result<()> {
    tracing::trace!("listen");
    let v4: JoinHandle<Result<()>> = spawn(async {
        let listener = TcpListener::bind("0.0.0.0:7145").await?;

        loop {
            let (socket, _) = listener.accept().await?;
            tracing::trace!("accept: {}", socket.peer_addr()?);
            tokio::spawn(async move {
                if let Err(e) = process(socket).await {
                    error!("{}", e);
                }
            });
        }
    });
    let v6: JoinHandle<Result<()>> = spawn(async {
        let listener = TcpListener::bind("[::]:7145").await?;

        loop {
            let (socket, _) = listener.accept().await?;
            tracing::trace!("accept: {}", socket.peer_addr()?);
            tokio::spawn(async move {
                if let Err(e) = process(socket).await {
                    error!("{}", e);
                }
            });
        }
    });
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
