use std::fmt::Debug;

use super::values::{AtomIpAddr, Flg1, Id, VExP};

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename = "pcp\n")]
pub struct Pcp(pub u32);

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename = "quit")]
pub struct Quit(pub u32);

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename = "helo")]
pub struct Helo {
    pub sid: Id,
    pub agnt: Option<String>,
    pub ver: Option<u32>,
    pub port: Option<u16>,
    pub ping: Option<u16>,
    pub bcid: Option<Id>,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename = "oleh")]
pub struct Oleh {
    pub sid: Id,
    pub agnt: Option<String>,
    pub ver: Option<u32>,
    pub rip: Option<AtomIpAddr>,
    pub port: Option<u16>,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Host {
    pub cid: Id,
    pub id: Id,
    #[serde(rename = "ip\0\0port")]
    pub ip_port: Vec<(AtomIpAddr, u16)>,
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
    pub upip: Option<AtomIpAddr>,
    pub uppt: Option<u32>, // WTF
    pub uphp: Option<u32>,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Info {
    pub name: String,
    pub bitr: u32,
    pub gnre: String,
    pub url: String,
    pub desc: String,
    pub cmnt: String,
    pub r#type: String,
    pub styp: String,
    pub sext: String,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Trck {
    pub titl: String,
    pub crea: String,
    pub url: String,
    pub albm: String,
}

#[derive(Debug, PartialEq, serde::Deserialize, serde::Serialize)]
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
