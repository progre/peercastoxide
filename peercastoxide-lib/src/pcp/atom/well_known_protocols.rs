use std::{
    fmt::Debug,
    io::{Read, Write},
    net::{IpAddr, SocketAddr, TcpStream},
    time::Duration,
};

use anyhow::{anyhow, bail, Result};
use tracing::debug;

use crate::pcp::atom::{
    self,
    future::AtomStreamReader,
    values::Id,
    well_known_atoms::{Helo, Oleh, Pcp, Quit},
};

fn leave_connection(stream: TcpStream) -> Result<()> {
    stream.set_nonblocking(true)?;
    let mut reader = AtomStreamReader::new(tokio::net::TcpStream::from_std(stream)?);
    tokio::spawn(tokio::time::timeout(Duration::from_secs(10), async move {
        let span = tracing::trace_span!("after_ping");
        let _guard = span.enter();
        loop {
            let res = reader
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
    Ok(())
}

fn ping(
    peer_addr: &SocketAddr,
    session_id: &Id,
    peer_session_id: &Id,
    timeout: Duration,
) -> Result<()> {
    tracing::trace!("ping start: {}", peer_addr);
    let mut stream = TcpStream::connect(peer_addr)?;
    stream.set_read_timeout(Some(timeout))?;
    stream.set_write_timeout(Some(timeout))?;

    let pcp = if peer_addr.is_ipv6() {
        Pcp(100)
    } else {
        Pcp(1)
    };
    atom::to_writer(&mut stream, pcp)?;

    let helo = Helo {
        sid: session_id.clone(),
        agnt: None,
        ver: None,
        port: None,
        ping: None,
        bcid: None,
    };
    atom::to_writer(&mut stream, helo)?;

    let oleh: Oleh = atom::from_reader(&mut stream)?;
    if &oleh.sid != peer_session_id {
        bail!("session id mismatch")
    }

    atom::to_writer(&mut stream, Quit(1000))?;

    tracing::trace!("ping succeeded: {}", peer_addr);

    leave_connection(stream)?;
    Ok(())
}

#[tracing::instrument]
pub fn handshake(
    stream: &mut (impl Read + Write + Debug),
    session_id: &Id,
    peer_ip_addr: IpAddr,
    agent_name: &'static str,
    ping_timeout: Duration,
) -> Result<()> {
    let pcp: Pcp = atom::from_reader(stream)?;
    if pcp.0 != 1 && pcp.0 != 100 {
        bail!("invalid atom")
    }

    let helo: Helo = atom::from_reader(stream)?;

    let pinged_port = 'block: {
        let Some(ping_port) = helo.ping else {
            break 'block None;
        };
        let peer_addr = SocketAddr::new(peer_ip_addr, ping_port);
        match ping(&peer_addr, session_id, &helo.sid, ping_timeout) {
            Ok(()) => Some(ping_port),
            Err(err) => {
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
    atom::to_writer(stream, oleh)?;

    tracing::trace!("handshake succeeded");
    Ok(())
}
