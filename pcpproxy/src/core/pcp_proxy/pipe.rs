use std::num::NonZeroU16;
use std::{borrow::Cow, net::Ipv4Addr};

use anyhow::Result;
use tokio::io::AsyncRead;
use tokio::net::tcp::OwnedWriteHalf;

use crate::core::utils::PipeError;
use crate::features::output::ndjson::NDJson;
use crate::features::pcp::atom::AtomChild;
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
) -> Vec<((usize, Ipv4Addr), (usize, NonZeroU16))> {
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
        .map(|(idx, atom)| (idx, atom.to_ipv4()));
    let port_indices = children
        .iter()
        .enumerate()
        .filter_map(|item| filter_map(PORT, item))
        .map(|(idx, atom)| (idx, atom.to_u16().try_into().unwrap()));
    ip_indices.zip(port_indices).collect()
}

fn replace_ipv4_port_pair(
    children: &mut [Atom],
    ipv4_idx: usize,
    port_idx: usize,
    ipv4: Ipv4Addr,
    port: NonZeroU16,
) {
    if let Atom::Child(ip_atom) = &mut children[ipv4_idx] {
        *ip_atom = AtomChild::ipv4(Cow::Borrowed(IP), ipv4);
    } else {
    }
    if let Atom::Child(port_atom) = &mut children[port_idx] {
        *port_atom = AtomChild::u16(Cow::Borrowed(PORT), port.into());
    } else {
    }
}

pub async fn pipe_pcp(
    incoming: impl AsyncRead + Unpin + Send + Sync,
    outgoing: OwnedWriteHalf,
    ipv4_addr_from_real_server: Ipv4Addr,
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
            Atom::Parent(parent) if parent.identifier() == HOST => {
                let indices = find_ip_port_pair_indices(parent.children());
                for ((ip_idx, replace_from_ip), (port_idx, replace_from_port)) in indices {
                    let replace_from = format!("{}:{}", replace_from_ip, replace_from_port);
                    let replace_to_port =
                        listen_for(ipv4_addr_from_real_server, replace_from.clone()).await;
                    replace_ipv4_port_pair(
                        parent.children_mut(),
                        ip_idx,
                        port_idx,
                        ipv4_addr_from_real_server,
                        replace_to_port,
                    );
                    let replace_to = format!("{}:{}", ipv4_addr_from_real_server, replace_to_port);
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
