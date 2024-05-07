use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use anyhow::{bail, Result};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    time::timeout,
};
use tracing::debug;

use crate::pcp::atom::{
    atom_stream::{AtomStreamReader, AtomStreamWriter},
    values::Id,
    well_known_atoms::{Helo, Oleh, Pcp, Quit},
};

async fn leave_connection(
    mut reader: AtomStreamReader<impl AsyncRead + Unpin + Send + Sync + 'static>,
) -> Result<()> {
    tokio::spawn(tokio::time::timeout(Duration::from_secs(10), async move {
        let span = tracing::trace_span!("after_ping");
        let _guard = span.enter();
        loop {
            let res = reader.read_unknown_atom().await;
            match res {
                Ok(atom) => tracing::trace!("({:#})", atom),
                Err(err) => {
                    debug!("{}", err);
                    return;
                }
            }
        }
    }));
    Ok(())
}

async fn ping(peer_addr: &SocketAddr, session_id: &Id, peer_session_id: &Id) -> Result<()> {
    tracing::trace!("ping start: {}", peer_addr);
    let (reader, writer) = TcpStream::connect(peer_addr).await?.into_split();
    let mut reader = AtomStreamReader::new(reader);
    let mut writer = AtomStreamWriter::new(writer);

    let pcp = if peer_addr.is_ipv6() {
        Pcp(100)
    } else {
        Pcp(1)
    };
    writer.write_atom(&pcp).await?;

    let helo = Helo {
        sid: session_id.clone(),
        agnt: None,
        ver: None,
        port: None,
        ping: None,
        bcid: None,
    };
    writer.write_atom(&helo).await?;

    let oleh: Oleh = reader.read_atom().await?;
    if &oleh.sid != peer_session_id {
        bail!("session id mismatch")
    }

    writer.write_atom(&Quit(1000)).await?;

    tracing::trace!("ping succeeded: {}", peer_addr);

    leave_connection(reader).await?;
    Ok(())
}

pub async fn handshake(
    reader: &mut AtomStreamReader<impl AsyncRead + Unpin + Send + Sync>,
    writer: &mut AtomStreamWriter<impl AsyncWrite + Unpin + Send + Sync>,
    session_id: &Id,
    peer_ip_addr: IpAddr,
    agent_name: &'static str,
    ping_timeout: Duration,
) -> Result<()> {
    let pcp: Pcp = reader.read_atom().await?;
    if pcp.0 != 1 && pcp.0 != 100 {
        bail!("invalid atom")
    }

    let helo: Helo = reader.read_atom().await?;

    let pinged_port = 'block: {
        let Some(ping_port) = helo.ping else {
            break 'block None;
        };
        let peer_addr = SocketAddr::new(peer_ip_addr, ping_port);
        let result = timeout(ping_timeout, ping(&peer_addr, session_id, &helo.sid)).await;
        match result.map_err(|elapsed| elapsed.into()) {
            Ok(Ok(())) => Some(ping_port),
            Ok(Err(err)) | Err(err) => {
                debug!("port 0 peer: {}", peer_ip_addr);
                debug!("reason: {}", err);
                Some(0)
            }
        }
    };
    let peer_port = pinged_port.or(helo.port).unwrap_or(0);

    let oleh = Oleh {
        sid: session_id.clone(),
        agnt: Some(agent_name.into()),
        ver: Some(1218),
        rip: Some(peer_ip_addr.into()),
        port: Some(peer_port),
    };
    writer.write_atom(&oleh).await?;

    tracing::trace!("handshake succeeded");
    Ok(())
}
