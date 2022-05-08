use std::{borrow::Cow, cmp::min, net::Ipv4Addr};

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::{net::tcp::OwnedReadHalf, sync::Mutex};

use crate::features::output::ndjson::NDJson;
use crate::features::pcp::atom::AtomChild;
use crate::features::pcp::{
    atom::Atom,
    atom_identifier::{HOST, IP, PORT},
    atom_stream::{AtomStreamReader, AtomStreamWriter},
};

use super::sub_servers::SubServers;

const RELAY_HOOK: bool = false;

pub fn big_vec<T: Default>(len: usize) -> Vec<T> {
    let mut buf = Vec::with_capacity(len);
    let remaining = buf.spare_capacity_mut();
    for item in remaining {
        item.write(Default::default());
    }
    unsafe { buf.set_len(len) };
    buf
}

pub async fn pipe_raw(
    mut incoming: OwnedReadHalf,
    mut outgoing: OwnedWriteHalf,
    output: NDJson,
) -> Result<()> {
    let mut buf = big_vec(1024 * 1024);
    let mut http11 = false;
    loop {
        let n = incoming.read(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }
        if !http11 {
            let buf2: &[u8] = &buf[0..n];
            let buf_str = String::from_utf8_lossy(&buf2[0..(min(buf2.len(), 100))]);
            if buf_str.starts_with("HTTP/1.1") {
                http11 = true;
            }
            output.output_raw(&buf_str);
        }
        outgoing.write_all(&buf[0..n]).await?;
    }
}

pub async fn pipe_pcp(
    incoming: OwnedReadHalf,
    outgoing: OwnedWriteHalf,
    sub_servers: &Mutex<SubServers>,
    output: NDJson,
) -> Result<()> {
    let mut atom_stream_reader = AtomStreamReader::new(incoming);
    let mut atom_stream_writer = AtomStreamWriter::new(outgoing);
    let mut host = false;
    let mut ip: Option<Ipv4Addr> = None;
    loop {
        let atom = if let Some(some) = atom_stream_reader.read().await? {
            some
        } else {
            return Ok(());
        };
        output.output(&atom);
        if RELAY_HOOK {
            if host {
                if ip.is_none() && atom.identifier() == IP {
                    if let Atom::Child(child) = &atom {
                        let current = child.to_ipv4();
                        if !current.is_private() {
                            ip = Some(current);
                            continue;
                        }
                    }
                } else if ip.is_some() && atom.identifier() == PORT {
                    if let Atom::Child(child) = &atom {
                        let port = child.to_u16();
                        let (hook_ip, hook_port) = sub_servers
                            .lock()
                            .await
                            .start_server(ip.unwrap(), port)
                            .await?;
                        ip = None;
                        host = false;

                        let atom = Atom::Child(AtomChild::ipv4(Cow::Borrowed(IP), hook_ip));
                        atom_stream_writer.write(&atom).await?;
                        let atom = Atom::Child(AtomChild::u16(Cow::Borrowed(PORT), hook_port));
                        atom_stream_writer.write(&atom).await?;
                        continue;
                    }
                }
            } else if atom.identifier() == HOST {
                host = true;
            }
        }
        atom_stream_writer.write(&atom).await?;
    }
}
