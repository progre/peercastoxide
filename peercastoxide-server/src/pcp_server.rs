use std::time::Duration;

use anyhow::{anyhow, Result};
use peercastoxide_lib::pcp::atom::{self, well_known_protocols::handshake, Id};
use tokio::{
    net::{TcpListener, TcpStream, ToSocketAddrs},
    select, spawn,
    task::{block_in_place, JoinHandle},
};
use tracing::error;

const AGENT_NAME: &str = concat!("PeerCastOxide/", env!("CARGO_PKG_VERSION"));

async fn process(stream: TcpStream) -> Result<()> {
    let session_id = Id(rand::random());
    let peer_addr = stream.peer_addr()?;

    let mut stream = block_in_place(move || -> Result<TcpStream> {
        let mut std_stream = stream.into_std()?;
        std_stream.set_nonblocking(false)?;
        std_stream.set_read_timeout(Some(Duration::from_secs(15)))?;
        std_stream.set_write_timeout(Some(Duration::from_secs(15)))?;

        handshake(
            &mut std_stream,
            &session_id,
            peer_addr.ip(),
            AGENT_NAME,
            Duration::from_secs(15),
        )?;

        Ok(TcpStream::from_std(std_stream)?)
    })?;

    let (read_half, _write_half) = stream.split();
    let mut read_stream = atom::future::AtomStreamReader::new(read_half);

    loop {
        let atom = read_stream
            .read_atom()
            .await
            .map_err(|e| anyhow!("failed: {:?}", e))?
            .ok_or_else(|| anyhow!("eos"))?;
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

pub async fn listen() -> anyhow::Result<()> {
    tracing::trace!("listen");
    let v4: JoinHandle<Result<()>> = spawn(accept_connenctions_loop("0.0.0.0:7145"));
    let v6: JoinHandle<Result<()>> = spawn(accept_connenctions_loop("[::]:7145"));
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
