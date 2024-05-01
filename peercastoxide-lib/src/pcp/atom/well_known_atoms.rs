use std::fmt::Debug;

use super::{AtomString, Flg1, Id, IpAddr, SocketAddr, VExP};

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename = "pcp\n")]
pub struct Pcp(pub u32);

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "quit\n")]
pub struct Quit(pub u32);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename = "helo")]
pub struct Helo {
    pub sid: Id,
    pub agnt: Option<AtomString>,
    pub ver: Option<u32>,
    pub port: Option<u16>,
    pub ping: Option<u16>,
    pub bcid: Option<Id>,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename = "oleh")]
pub struct Oleh {
    pub sid: Id,
    pub agnt: Option<AtomString>,
    pub ver: Option<u32>,
    pub rip: Option<IpAddr>,
    pub port: Option<u16>,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename = "host")]
pub struct Host {
    pub cid: Id,
    pub id: Id,
    #[serde(rename = "ip\0\0port")]
    pub ip_port: Vec<SocketAddr>,
    pub numl: u32,
    pub numr: u32,
    pub uptm: u32,
    pub ver: u32,
    pub vevp: u32,
    pub vexp: VExP,
    pub vexn: u16,
    pub flg1: Flg1,
    pub oldp: u32,
    pub newp: u32,
    pub upip: Option<IpAddr>,
    pub uppt: Option<u32>, // WTF
    pub uphp: Option<u32>,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename = "info")]
pub struct Info {
    pub name: AtomString,
    pub bitr: u32,
    pub gnre: AtomString,
    pub url: AtomString,
    pub desc: AtomString,
    pub cmnt: AtomString,
    pub r#type: AtomString,
    pub styp: AtomString,
    pub sext: AtomString,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename = "trck")]
pub struct Trck {
    pub titl: AtomString,
    pub crea: AtomString,
    pub url: AtomString,
    pub albm: AtomString,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename = "chan")]
pub struct Chan {
    pub id: Id,
    pub bcid: Id,
    pub info: Info,
    pub trck: Trck,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename = "bcst")]
pub struct Bcst {
    pub grp: u8,
    pub hops: u8,
    pub ttl: u8,
    pub from: Id,
    pub vers: u32,
    pub vrvp: u32,
    pub vexp: VExP,
    pub vexn: u16,
    pub chan: Chan,
    pub host: Host,
}
