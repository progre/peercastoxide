use std::{
    fmt::{Debug, Display, Formatter},
    net::IpAddr,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Eq, Hash, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Id(pub [u8; 16]);

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.iter().try_for_each(|&x| write!(f, "{:02x}", x))
    }
}

impl Debug for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(transparent)]
pub struct Flg1(pub u8);

impl Flg1 {
    pub fn tracker(&self) -> bool {
        self.0 & 1 << 0 != 0
    }
    pub fn relay(&self) -> bool {
        self.0 & 1 << 1 != 0
    }
    pub fn direct(&self) -> bool {
        self.0 & 1 << 2 != 0
    }
    pub fn push(&self) -> bool {
        self.0 & 1 << 3 != 0
    }
    pub fn recv(&self) -> bool {
        self.0 & 1 << 4 != 0
    }
    pub fn cin(&self) -> bool {
        self.0 & 1 << 5 != 0
    }
    pub fn private(&self) -> bool {
        self.0 & 1 << 6 != 0
    }
}

impl Debug for Flg1 {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let tracker = self.0 & 1 << 0 != 0;
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
            if tracker { "TR" } else { "tr" },
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

#[derive(PartialEq)]
pub struct AtomIpAddr(pub IpAddr);

impl Debug for AtomIpAddr {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<IpAddr> for AtomIpAddr {
    fn from(ip_addr: IpAddr) -> Self {
        AtomIpAddr(ip_addr)
    }
}

impl From<AtomIpAddr> for IpAddr {
    fn from(val: AtomIpAddr) -> Self {
        val.0
    }
}

impl From<[u8; 4]> for AtomIpAddr {
    fn from(octets: [u8; 4]) -> Self {
        AtomIpAddr(IpAddr::V4(std::net::Ipv4Addr::from(octets)))
    }
}

impl From<[u8; 16]> for AtomIpAddr {
    fn from(octets: [u8; 16]) -> Self {
        AtomIpAddr(IpAddr::V6(std::net::Ipv6Addr::from(octets)))
    }
}

impl<'a> Deserialize<'a> for AtomIpAddr {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        let vec: Vec<_> = <Vec<u8>>::deserialize(deserializer)?
            .into_iter()
            .rev()
            .collect();
        match vec.len() {
            4 => {
                let mut octets = [0u8; 4];
                octets.copy_from_slice(&vec);
                Ok(IpAddr::from(octets).into())
            }
            16 => {
                let mut octets = [0u8; 16];
                octets.copy_from_slice(&vec);
                Ok(IpAddr::from(octets).into())
            }
            _ => Err(serde::de::Error::custom("invalid length")),
        }
    }
}

impl Serialize for AtomIpAddr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let vec: Vec<_> = match &self.0 {
            IpAddr::V4(v4) => v4.octets().into_iter().rev().collect(),
            IpAddr::V6(v6) => v6.octets().into_iter().rev().collect(),
        };
        vec.serialize(serializer)
    }
}
