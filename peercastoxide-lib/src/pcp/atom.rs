mod deserializer;
pub mod future;
mod serializer;
mod unknown;

pub mod well_known_atoms;
pub mod well_known_protocols;

use std::borrow::Cow;
use std::ffi::{CStr, CString, NulError};
use std::fmt::{Debug, Formatter};
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

use anyhow::{anyhow, Result};

pub use deserializer::from_reader;
pub use serializer::to_writer;

fn to_string_without_zero_padding(string: &[u8]) -> String {
    String::from_utf8_lossy(string)
        .trim_end_matches('\0')
        .to_string()
}

#[derive(Clone, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Id(pub [u8; 16]);

impl Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.iter().try_for_each(|&x| write!(f, "{:02x}", x))
    }
}

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct AtomString(pub Vec<u8>);

impl AtomString {
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.0)
    }
}

impl FromStr for AtomString {
    type Err = NulError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(AtomString(CString::new(s)?.as_bytes_with_nul().to_vec()))
    }
}

impl Debug for AtomString {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match CStr::from_bytes_until_nul(&self.0) {
            Ok(str) => write!(f, "{:?}", str),
            Err(_) => self.0.iter().try_for_each(|&x| write!(f, "{:02x}", x)),
        }
    }
}

#[derive(Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Flg1(pub u8);

impl Debug for Flg1 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let tr = self.0 & 1 << 0 != 0;
        let relay = self.0 & 1 << 1 != 0;
        let direct = self.0 & 1 << 2 != 0;
        let push = self.0 & 1 << 3 != 0;
        let recv = self.0 & 1 << 4 != 0;
        let cin = self.0 & 1 << 5 != 0;
        let private = self.0 & 1 << 6 != 0;
        let unused = self.0 & 1 << 7 != 0;
        write!(
            f,
            "{}|{}|{}|{}|{}|{}|{}|{}",
            if tr { "TR" } else { "tr" },
            if relay { "RE" } else { "re" },
            if direct { "DI" } else { "di" },
            if push { "PU" } else { "pu" },
            if recv { "RE" } else { "re" },
            if cin { "CI" } else { "ci" },
            if private { "PR" } else { "pr" },
            if unused { "!" } else { "_" },
        )
    }
}

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct VExP(pub [u8; 2]);

impl Debug for VExP {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{:?}", String::from_utf8_lossy(&self.0))
    }
}

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum IpAddr {
    V4([u8; 4]),
    V6([u8; 16]),
}

impl IpAddr {
    pub fn to_std(&self) -> Result<std::net::IpAddr> {
        match self {
            IpAddr::V4(v4) => {
                let vec: Vec<_> = v4.iter().cloned().rev().collect();
                let octets: [_; 4] = vec.try_into().map_err(|_| anyhow!("size mismatch"))?;
                Ok(std::net::IpAddr::V4(Ipv4Addr::from(octets)))
            }
            IpAddr::V6(v6) => {
                let vec: Vec<_> = v6.iter().cloned().rev().collect();
                let octets: [_; 16] = vec.try_into().map_err(|_| anyhow!("size mismatch"))?;
                Ok(std::net::IpAddr::V6(Ipv6Addr::from(octets)))
            }
        }
    }

    pub fn from_std(ip_addr: &std::net::IpAddr) -> Self {
        match ip_addr {
            std::net::IpAddr::V4(v4) => {
                let vec: Vec<_> = v4.octets().iter().cloned().rev().collect();
                let mut buf = [0u8; 4];
                buf.copy_from_slice(&vec);
                IpAddr::V4(buf)
            }
            std::net::IpAddr::V6(v6) => {
                let vec: Vec<_> = v6.octets().iter().cloned().rev().collect();
                let mut buf = [0u8; 16];
                buf.copy_from_slice(&vec);
                IpAddr::V6(buf)
            }
        }
    }
}

impl Debug for IpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let Ok(ip_addr) = self.to_std() else {
            return match self {
                IpAddr::V4(v4) => write!(f, "{:?}", v4),
                IpAddr::V6(v6) => write!(f, "{:?}", v6),
            };
        };
        write!(f, "{:?}", ip_addr)
    }
}

#[derive(PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SocketAddr(pub IpAddr, pub u16);

impl SocketAddr {
    pub fn to_std(&self) -> Result<std::net::SocketAddr> {
        Ok(std::net::SocketAddr::new(self.0.to_std()?, self.1))
    }
}

impl Debug for SocketAddr {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let Ok(addr) = self.to_std() else {
            return write!(f, "{:?}", self.0);
        };
        write!(f, "{:?}", addr)
    }
}

fn is_grouped_atoms(identifier: &str) -> bool {
    identifier.len() / 4 >= 2
}

fn to_grouped_atoms(identifier: &str) -> Vec<[u8; 4]> {
    identifier
        .bytes()
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|x| {
            let mut buf = [0u8; 4];
            buf.copy_from_slice(x);
            buf
        })
        .collect()
}
