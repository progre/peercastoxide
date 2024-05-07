use std::net::SocketAddr;

mod bool_as_number {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(x: &bool, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if *x {
            s.serialize_u8(1)
        } else {
            s.serialize_u8(0)
        }
    }

    pub fn deserialize<'de, D>(d: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        let num = u8::deserialize(d)?;
        match num {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(serde::de::Error::custom("expected 0 or 1")),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Servent {
    /// seconds
    #[serde(rename = "@uptime")]
    pub uptime: u64,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Bandwidth {
    #[serde(rename = "@out")]
    pub out: u32,
    #[serde(rename = "@in")]
    pub r#in: u32,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Connections {
    #[serde(rename = "@total")]
    pub total: u32,
    #[serde(rename = "@relays")]
    pub relays: u32,
    #[serde(rename = "@direct")]
    pub direct: u32,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Host {
    #[serde(rename = "@ip")]
    pub ip: Option<SocketAddr>,
    #[serde(rename = "@hops")]
    pub hops: u8,
    #[serde(rename = "@listeners")]
    pub listeners: u32,
    #[serde(rename = "@relays")]
    pub relays: u32,
    #[serde(rename = "@uptime")]
    pub uptime: u32,
    #[serde(rename = "@push", with = "bool_as_number")]
    pub push: bool,
    #[serde(rename = "@relay", with = "bool_as_number")]
    pub relay: bool,
    #[serde(rename = "@direct", with = "bool_as_number")]
    pub direct: bool,
    #[serde(rename = "@cin", with = "bool_as_number")]
    pub cin: bool,
    #[serde(rename = "@stable")]
    pub stable: u32,
    #[serde(rename = "@version")]
    pub version: u32,
    #[serde(rename = "@update")]
    pub update: u64,
    #[serde(rename = "@tracker", with = "bool_as_number")]
    pub tracker: bool,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Hits {
    #[serde(rename = "@hosts")]
    pub hosts: u32,
    #[serde(rename = "@listeners")]
    pub listeners: u32,
    #[serde(rename = "@relays")]
    pub relays: u32,
    #[serde(rename = "@firewalled", with = "bool_as_number")]
    pub firewalled: bool,
    #[serde(rename = "@closest")]
    pub closest: u32,
    #[serde(rename = "@furthest")]
    pub furthest: u8,
    #[serde(rename = "@newest")]
    pub newest: u32,

    #[serde(default)]
    pub host: Vec<Host>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Track {
    #[serde(rename = "@title")]
    pub title: String,
    /// crea(tor) on atom
    #[serde(rename = "@artist")]
    pub artist: String,
    #[serde(rename = "@album")]
    pub album: String,
    #[serde(rename = "@genre")]
    pub genre: String,
    /// alb(u)m on atom
    #[serde(rename = "@contact")]
    pub contact: String,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Channel {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@bitrate")]
    pub bitrate: u32,
    #[serde(rename = "@type")]
    pub r#type: String,
    #[serde(rename = "@genre")]
    pub genre: String,
    #[serde(rename = "@desc")]
    pub desc: String,
    #[serde(rename = "@url")]
    pub url: String,
    #[serde(rename = "@uptime")]
    pub uptime: u64,
    #[serde(rename = "@comment")]
    pub comment: String,
    /// Not on PeerCastStation
    #[serde(rename = "@skips", skip_serializing_if = "Option::is_none")]
    pub skips: Option<u32>,
    /// PeerCastStation only
    #[serde(rename = "@skip", skip_serializing_if = "Option::is_none")]
    pub skip: Option<u32>,
    /// Time since this peer received it
    #[serde(rename = "@age")]
    pub age: u64,
    #[serde(rename = "@bcflags")]
    pub bcflags: u32,

    pub hits: Hits,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relay: Option<()>,
    pub track: Track,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ChannelsRelayed {
    #[serde(rename = "@total")]
    pub total: u32,

    #[serde(default)]
    pub channel: Vec<Channel>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct ChannelsFound {
    #[serde(rename = "@total")]
    pub total: u32,

    #[serde(default)]
    pub channel: Vec<Channel>,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Peercast {
    /// PeerCastStation only
    #[serde(rename = "@session", skip_serializing_if = "Option::is_none")]
    pub session: Option<String>,

    pub servent: Servent,
    pub bandwidth: Bandwidth,
    pub connections: Connections,
    pub channels_relayed: ChannelsRelayed,
    pub channels_found: ChannelsFound,
    /// Not on PeerCastStation
    pub host_cache: Option<()>,
}
