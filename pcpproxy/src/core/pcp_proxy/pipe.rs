use std::{borrow::Cow, net::Ipv4Addr};

use anyhow::Result;
use tokio::io::AsyncRead;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;

use crate::core::utils::PipeError;
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

pub async fn pipe_pcp(
    incoming: impl AsyncRead + Unpin + Send + Sync,
    outgoing: OwnedWriteHalf,
    sub_servers: &Mutex<SubServers>,
    output: &NDJson,
) -> Result<(), PipeError> {
    let mut atom_stream_reader = AtomStreamReader::new(incoming);
    let mut atom_stream_writer = AtomStreamWriter::new(outgoing);
    let mut host = false;
    let mut ip: Option<Ipv4Addr> = None;
    loop {
        let atom = if let Some(some) = atom_stream_reader
            .read()
            .await
            .map_err(PipeError::ByIncoming)?
        {
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
                            .await
                            .unwrap();
                        ip = None;
                        host = false;

                        let atom = Atom::Child(AtomChild::ipv4(Cow::Borrowed(IP), hook_ip));
                        atom_stream_writer.write(&atom).await.unwrap();
                        let atom = Atom::Child(AtomChild::u16(Cow::Borrowed(PORT), hook_port));
                        atom_stream_writer.write(&atom).await.unwrap();
                        continue;
                    }
                }
            } else if atom.identifier() == HOST {
                host = true;
            }
        }
        atom_stream_writer
            .write(&atom)
            .await
            .map_err(PipeError::ByOutgoing)?;
    }
}
