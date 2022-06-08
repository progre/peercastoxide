use std::borrow::Cow;
use std::net::{IpAddr, SocketAddr};
use std::num::NonZeroU16;

use anyhow::Result;
use tokio::io::AsyncRead;
use tokio::net::tcp::OwnedWriteHalf;

use crate::core::utils::PipeError;
use crate::features::output::ndjson::NDJson;
use crate::features::pcp::atom::AtomChild;
use crate::features::pcp::atom_identifier::{BCST, HELO};
use crate::features::pcp::{
    atom::Atom,
    atom_identifier::{HOST, IP, PORT},
    atom_stream::{AtomStreamReader, AtomStreamWriter},
};
use crate::features::real_server_listener::listen_for::listen_for;

pub fn big_vec<T: Default>(len: usize) -> Vec<T> {
    let mut buf = Vec::with_capacity(len);
    let remaining = buf.spare_capacity_mut();
    for item in remaining {
        item.write(Default::default());
    }
    unsafe { buf.set_len(len) };
    buf
}

fn find_ip_port_pair_indices<'a>(
    children: &'a [Atom],
) -> Vec<((usize, IpAddr), (usize, NonZeroU16))> {
    let filter_map = |identifier: &[u8; 4], (i, x): (usize, &'a Atom)| {
        if x.identifier() != identifier {
            return None;
        }
        let child = if let Atom::Child(child) = &x {
            child
        } else {
            return None;
        };
        Some((i, child))
    };
    let ip_indices = children
        .iter()
        .enumerate()
        .filter_map(|item| filter_map(IP, item))
        .map(|(idx, atom)| (idx, atom.to_ip()));
    let port_indices = children
        .iter()
        .enumerate()
        .filter_map(|item| filter_map(PORT, item))
        .map(|(idx, atom)| (idx, atom.to_u16().try_into().unwrap()));
    ip_indices.zip(port_indices).collect()
}

fn replace_ip_port_pair(
    children: &mut [Atom],
    ip_idx: usize,
    port_idx: usize,
    ip: IpAddr,
    port: NonZeroU16,
) {
    if let Atom::Child(ip_atom) = &mut children[ip_idx] {
        *ip_atom = AtomChild::ip(Cow::Borrowed(IP), ip);
    }
    if let Atom::Child(port_atom) = &mut children[port_idx] {
        *port_atom = AtomChild::u16(Cow::Borrowed(PORT), port.into());
    }
}

pub async fn pipe_pcp(
    incoming: impl AsyncRead + Unpin + Send + Sync,
    outgoing: OwnedWriteHalf,
    real_server_port: NonZeroU16,
    ip_addr_from_real_server: IpAddr,
    listen_port: NonZeroU16,
    output: &NDJson,
) -> Result<(), PipeError> {
    let mut atom_stream_reader = AtomStreamReader::new(incoming);
    let mut atom_stream_writer = AtomStreamWriter::new(outgoing);
    loop {
        let mut atom = if let Some(some) = atom_stream_reader
            .read()
            .await
            .map_err(PipeError::ByIncoming)?
        {
            some
        } else {
            return Ok(());
        };
        output.output(&atom);

        match &mut atom {
            Atom::Parent(parent) if parent.identifier() == BCST => {
                for child in parent
                    .children_mut()
                    .iter_mut()
                    .filter(|x| x.identifier() == HOST)
                    .filter_map(|x| {
                        if let Atom::Parent(parent) = x {
                            Some(parent)
                        } else {
                            None
                        }
                    })
                    .flat_map(|x| x.children_mut())
                    .filter(|x| x.identifier() == PORT)
                    .filter_map(|x| {
                        if let Atom::Child(child) = x {
                            Some(child)
                        } else {
                            None
                        }
                    })
                    .filter(|x| x.to_u16() == real_server_port.get())
                {
                    *child = AtomChild::u16(Cow::Borrowed(PORT), listen_port.get());
                    output.info(&format!(
                        "Proxy: Replaced {} with {}",
                        real_server_port, listen_port
                    ));
                }
            }
            Atom::Parent(parent) if parent.identifier() == HELO => {
                if parent.children().iter().all(|x| x.identifier() != PORT) {
                    parent.children_mut().push(Atom::Child(AtomChild::u16(
                        Cow::Borrowed(PORT),
                        listen_port.get(),
                    )));
                    output.info(&format!("Proxy: Append AtomChild(port, {})", listen_port));
                } else {
                    for child in parent
                        .children_mut()
                        .iter_mut()
                        .filter(|x| x.identifier() == PORT)
                        .filter_map(|x| {
                            if let Atom::Child(child) = x {
                                Some(child)
                            } else {
                                None
                            }
                        })
                        .filter(|x| x.to_u16() == real_server_port.get())
                    {
                        *child = AtomChild::u16(Cow::Borrowed(PORT), listen_port.get());
                        output.info(&format!(
                            "Proxy: Replaced {} with {}",
                            real_server_port, listen_port
                        ));
                    }
                }
            }
            Atom::Parent(parent) if parent.identifier() == HOST => {
                let indices = find_ip_port_pair_indices(parent.children());
                for ((ip_idx, replace_from_ip), (port_idx, replace_from_port)) in indices {
                    let replace_from = SocketAddr::new(replace_from_ip, replace_from_port.get());
                    let replace_to_port = listen_for(
                        real_server_port,
                        ip_addr_from_real_server,
                        listen_port,
                        replace_from,
                    )
                    .await;
                    replace_ip_port_pair(
                        parent.children_mut(),
                        ip_idx,
                        port_idx,
                        ip_addr_from_real_server,
                        replace_to_port,
                    );
                    let replace_to = format!("{}:{}", ip_addr_from_real_server, replace_to_port);
                    output.info(&format!(
                        "Proxy: Replaced {} with {}",
                        replace_from, replace_to
                    ));
                }
            }
            _ => {}
        }

        atom_stream_writer
            .write(&atom)
            .await
            .map_err(PipeError::ByOutgoing)?;
    }
}
