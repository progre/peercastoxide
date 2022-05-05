use crate::pcp::{atom::Atom, atom_stream::AtomStreamWriter};
use crate::pcp::{atom::AtomContent, atom_stream::AtomStreamReader};
use crate::{
    console::{console_color::ConsoleColor, printer::PcpPrinter},
    pcp::atom_identifier::{HOST, IP},
};
use anyhow::Result;
use log::*;
use std::{cmp::min, net::Ipv4Addr};
use tokio::net::tcp::OwnedReadHalf;
use tokio::net::tcp::OwnedWriteHalf;
use tokio::prelude::*;

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

fn start_server() {}

pub async fn pipe_pcp(
    from: OwnedReadHalf,
    to: OwnedWriteHalf,
    local_addr: Ipv4Addr,
    color: ConsoleColor,
) -> Result<()> {
    let mut printer = PcpPrinter::new(&color);
    let mut atom_stream_reader = AtomStreamReader::new(from);
    let mut atom_stream_writer = AtomStreamWriter::new(to);
    let mut host = false;
    loop {
        let atom = if let Some(some) = atom_stream_reader.read().await? {
            some
        } else {
            return Ok(());
        };
        printer.print(&atom);
        if host && atom.identifier() == IP {
            if let AtomContent::Child(child) = atom.content() {
                if !child.to_ipv4().is_private() {
                    let atom = Atom::ipv4(IP, local_addr);
                    atom_stream_writer.write(&atom).await?;
                }
            }
        } else if atom.identifier() == HOST {
            host = true;
        }
        atom_stream_writer.write(&atom).await?;
    }
}
