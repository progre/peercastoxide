use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};

use anyhow::{anyhow, bail, Error, Result};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
    spawn,
    time::timeout,
};
use tracing::debug;

use crate::pcp::atom::future::{
    custom_atom::CustomAtom,
    well_known_atoms::{helo_minimum, oleh, pcp_ipv4, pcp_ipv6},
    well_known_identifiers::*,
    AtomStreamReader, AtomStreamWriter,
};

async fn ping(
    peer_addr: &SocketAddr,
    session_id: &[u8; 16],
    peer_session_id: &[u8; 16],
) -> Result<()> {
    tracing::trace!("ping start: {}", peer_addr);
    let socket = TcpStream::connect(peer_addr).await?;
    let (read_half, write_half) = socket.into_split();
    let mut read_stream = AtomStreamReader::new(read_half);
    let mut write_stream = AtomStreamWriter::new(write_half);

    let pcp = if peer_addr.is_ipv6() {
        pcp_ipv6()
    } else {
        pcp_ipv4()
    };
    write_stream.write_atom(&pcp).await?;
    write_stream.write_atom(&helo_minimum(*session_id)).await?;

    let oleh = read_stream
        .read_atom()
        .await?
        .ok_or_else(|| anyhow!("end of stream"))?;
    if oleh.identifier() != OLEH {
        bail!("expected oleh but got {}", oleh)
    }
    let CustomAtom::Parent(oleh) = oleh else {
        bail!("expected oleh as parent but it's child")
    };
    let Some(CustomAtom::Child(sid)) = oleh.children().iter().find(|x| x.identifier() == SID)
    else {
        bail!("sid not found")
    };
    if sid.data() != peer_session_id {
        bail!("session id mismatch")
    }

    write_stream
        .write_atom(&CustomAtom::u16(QUIT, 1000))
        .await?;
    spawn(timeout(Duration::from_secs(10), async move {
        let span = tracing::trace_span!("after_ping");
        let _guard = span.enter();
        loop {
            let res = read_stream
                .read_atom()
                .await
                .and_then(|x| x.ok_or_else(|| anyhow!("end of stream")));
            match res {
                Ok(atom) => tracing::trace!("({:#})", atom),
                Err(err) => {
                    debug!("{}", err);
                    return;
                }
            }
        }
    }));
    tracing::trace!("ping succeeded: {}", peer_addr);
    Ok(())
}

pub async fn handshake<R, W>(
    agent_name: &str,
    peer_ip_addr: IpAddr,
    read_stream: &mut AtomStreamReader<R>,
    write_stream: &mut AtomStreamWriter<W>,
    session_id: &[u8; 16],
    ping_timeout: Duration,
) -> Result<()>
where
    R: AsyncRead + Unpin + Send + Sync,
    W: AsyncWrite + Unpin + Send + Sync,
{
    tracing::trace!("handshake start");
    let pcp = read_stream
        .read_atom()
        .await?
        .ok_or_else(|| anyhow!("end of stream"))?;
    tracing::trace!("{:#}", pcp);
    if pcp != pcp_ipv4() && pcp != pcp_ipv6() {
        bail!("invalid atom")
    }

    let helo = read_stream
        .read_atom()
        .await?
        .ok_or_else(|| anyhow!("end of stream"))?;
    tracing::trace!("{:#}", helo);
    if helo.identifier() != HELO {
        let identifier = helo.to_identifier_string();
        bail!("expected helo but got {}", identifier)
    }
    let CustomAtom::Parent(helo) = helo else {
        bail!("expected helo as parent but it's child")
    };
    let peer_sid: [u8; 16] = {
        let Some(sid) = helo.children().iter().find(|x| x.identifier() == SID) else {
            bail!("sid not found")
        };
        let CustomAtom::Child(sid) = sid else {
            bail!("expected sid as child but it's parent")
        };
        match sid.data().try_into() {
            Ok(sid) => sid,
            Err(err) => bail!("invalid sid length ({:?})", err),
        }
    };
    let ping_port = 'block: {
        let Some(ping_atom) = helo.children().iter().find(|x| x.identifier() == PING) else {
            break 'block None;
        };
        let CustomAtom::Child(ping_atom) = ping_atom else {
            bail!("expected ping as child but it's parent")
        };
        match ping_atom.to_u16() {
            Ok(ping) => Some(ping),
            Err(err) => bail!("invalid ping atom (reason: {}): {}", err, ping_atom),
        }
    };
    let peer_port = 'block: {
        let Some(port) = helo.children().iter().find(|x| x.identifier() == PORT) else {
            break 'block 0;
        };
        let CustomAtom::Child(port) = port else {
            bail!("expected port as child but it's parent")
        };
        match port.to_u16() {
            Ok(port) => port,
            Err(err) => bail!("invalid port atom (reason: {}): {}", err, port),
        }
    };

    let pinged_port = 'block: {
        let Some(ping_port) = ping_port else {
            break 'block None;
        };
        let peer_addr = SocketAddr::new(peer_ip_addr, ping_port);
        let timeout_future = timeout(ping_timeout, ping(&peer_addr, session_id, &peer_sid));
        let Err(err) = timeout_future.await.map_err(Error::new).and_then(|x| x) else {
            break 'block Some(ping_port);
        };
        debug!("port 0 peer: {}", peer_ip_addr);
        debug!("reason: {}", err);
        Some(0)
    };

    let peer_port = pinged_port.unwrap_or(peer_port);

    let peer_addr = SocketAddr::new(peer_ip_addr, peer_port);
    let oleh = oleh(*session_id, agent_name, peer_addr);
    write_stream.write_atom(&oleh).await?;

    tracing::trace!("handshake succeeded");
    Ok(())
}
