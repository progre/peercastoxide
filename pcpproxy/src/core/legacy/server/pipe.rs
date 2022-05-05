use std::{borrow::Cow, cmp::min, net::Ipv4Addr};

use anyhow::Result;
use log::*;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::prelude::*;
use tokio::{net::tcp::OwnedReadHalf, sync::Mutex};

use crate::{
    core::legacy::console::{console_color::ConsoleColor, printer::PcpPrinter},
    features::pcp::{
        atom::{Atom, AtomChild},
        atom_identifier::{HOST, IP, PORT},
        atom_stream::{AtomStreamReader, AtomStreamWriter},
    },
};

use super::sub_servers::SubServers;

const RELAY_HOOK: bool = false;

pub fn big_vec<T>(len: usize) -> Vec<T> {
    let mut buf = Vec::with_capacity(len);
    unsafe { buf.set_len(len) };
    buf
}

fn to_ascii_str(buf: &[u8]) -> String {
    buf.iter()
        .map(|&x| {
            (if 0x20 <= x && x <= 0x7e {
                x as char
            } else {
                '�'
            })
            .to_string()
        })
        .collect::<Vec<_>>()
        .join("")
}

pub async fn pipe(
    mut from: OwnedReadHalf,
    mut to: OwnedWriteHalf,
    color: ConsoleColor,
) -> Result<()> {
    let mut buf = big_vec(1024 * 1024);
    let mut http11 = false;
    loop {
        let n = from.read(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }
        if !http11 {
            let buf2: &[u8] = &buf[0..n];
            let buf_str = to_ascii_str(&buf2[0..(min(buf2.len(), 100))]);
            if buf_str.starts_with("HTTP/1.1") {
                http11 = true;
            }
            trace!("{}{}{}", color.header(), &buf_str, color.footer(),);
        }
        to.write(&buf[0..n]).await?;
    }
}

pub async fn pipe_pcp(
    from: OwnedReadHalf,
    to: OwnedWriteHalf,
    sub_servers: &Mutex<SubServers>,
    color: ConsoleColor,
) -> Result<()> {
    let mut printer = PcpPrinter::new(&color);
    let mut atom_stream_reader = AtomStreamReader::new(from);
    let mut atom_stream_writer = AtomStreamWriter::new(to);
    let mut host = false;
    let mut ip: Option<Ipv4Addr> = None;
    loop {
        let atom = if let Some(some) = atom_stream_reader.read().await? {
            some
        } else {
            return Ok(());
        };
        printer.print(&atom);
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
