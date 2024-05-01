mod branch_serializer;
mod count_children;
mod data_serializer;
mod grouped_atoms;
mod helpers;
mod root_serializer;
mod serialize_parent_struct;

use std::{fmt::Display, io::Write};

use anyhow::Result;
use serde::Serialize;

use self::{count_children::count_children, root_serializer::RootSerializer};

#[derive(Debug, thiserror::Error)]
pub enum AtomSerializeError {
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    Serde(String),
}

impl serde::ser::Error for AtomSerializeError {
    fn custom<T: Display>(msg: T) -> Self {
        AtomSerializeError::Serde(msg.to_string())
    }
}

pub fn to_writer(writer: impl Write, value: impl Serialize) -> Result<(), AtomSerializeError> {
    value.serialize(RootSerializer::new(writer, count_children(&value)?))
}

#[cfg(test)]
mod tests {
    use std::{io::Cursor, str::FromStr};

    use crate::pcp::atom::{
        self,
        well_known_atoms::{Bcst, Chan, Helo, Host, Info, Oleh, Pcp, Trck},
        AtomString, Flg1, Id, IpAddr, SocketAddr, VExP,
    };

    #[test]
    fn test_to_writer_pcp() {
        let before = Pcp(1);
        let mut buf = Vec::new();
        atom::serializer::to_writer(&mut buf, &before).unwrap();
        let after: Pcp = atom::deserializer::from_reader(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn test_to_writer_helo_minimum() {
        let before = Helo {
            sid: Id([1; 16]),
            agnt: None,
            ver: None,
            port: None,
            ping: None,
            bcid: None,
        };
        let mut buf = Vec::new();
        atom::serializer::to_writer(&mut buf, &before).unwrap();
        let after: Helo = atom::deserializer::from_reader(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn test_to_writer_helo_full() {
        let before = Helo {
            sid: Id([1; 16]),
            agnt: Some(AtomString::from_str("agent").unwrap()),
            ver: Some(2),
            port: Some(3),
            ping: Some(4),
            bcid: Some(Id([5; 16])),
        };
        let mut buf = Vec::new();
        atom::serializer::to_writer(&mut buf, &before).unwrap();
        let after: Helo = atom::deserializer::from_reader(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn test_to_writer_oleh() {
        let before = Oleh {
            sid: Id([1; 16]),
            agnt: Some(AtomString::from_str("agent").unwrap()),
            rip: Some(IpAddr::V4([1, 2, 3, 4])),
            port: Some(5),
            ver: Some(7),
        };
        let mut buf = Vec::new();
        atom::serializer::to_writer(&mut buf, &before).unwrap();
        let after: Oleh = atom::deserializer::from_reader(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn test_to_writer_chan() {
        let before = Chan {
            id: Id([2; 16]),
            bcid: Id([3; 16]),
            info: Info {
                name: AtomString::from_str("name").unwrap(),
                bitr: 20,
                gnre: AtomString::from_str("genre").unwrap(),
                url: AtomString::from_str("url").unwrap(),
                desc: AtomString::from_str("description").unwrap(),
                cmnt: AtomString::from_str("comment").unwrap(),
                r#type: AtomString::from_str("type").unwrap(),
                styp: AtomString::from_str("subtype").unwrap(),
                sext: AtomString::from_str("secondary_ext").unwrap(),
            },
            trck: Trck {
                titl: AtomString::from_str("title").unwrap(),
                crea: AtomString::from_str("creator").unwrap(),
                url: AtomString::from_str("url").unwrap(),
                albm: AtomString::from_str("album").unwrap(),
            },
        };
        let mut buf = Vec::new();
        atom::serializer::to_writer(&mut buf, &before).unwrap();
        let after: Chan = atom::deserializer::from_reader(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn test_to_writer_bcst() {
        let before = Bcst {
            grp: 1,
            hops: 2,
            ttl: 3,
            from: Id([1; 16]),
            vers: 4,
            vrvp: 5,
            vexp: VExP([b'V', b'P']),
            vexn: 6,
            chan: Chan {
                id: Id([2; 16]),
                bcid: Id([3; 16]),
                info: Info {
                    name: AtomString::from_str("name").unwrap(),
                    bitr: 20,
                    gnre: AtomString::from_str("genre").unwrap(),
                    url: AtomString::from_str("url").unwrap(),
                    desc: AtomString::from_str("description").unwrap(),
                    cmnt: AtomString::from_str("comment").unwrap(),
                    r#type: AtomString::from_str("type").unwrap(),
                    styp: AtomString::from_str("subtype").unwrap(),
                    sext: AtomString::from_str("secondary_ext").unwrap(),
                },
                trck: Trck {
                    titl: AtomString::from_str("title").unwrap(),
                    crea: AtomString::from_str("creator").unwrap(),
                    url: AtomString::from_str("url").unwrap(),
                    albm: AtomString::from_str("album").unwrap(),
                },
            },
            host: Host {
                cid: Id([4; 16]),
                id: Id([5; 16]),
                ip_port: vec![
                    SocketAddr(IpAddr::V4([1, 2, 3, 4]), 5),
                    SocketAddr(
                        IpAddr::V6([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
                        6,
                    ),
                ],
                numl: 7,
                numr: 8,
                uptm: 9,
                ver: 10,
                vevp: 11,
                vexp: VExP([b'V', b'P']),
                vexn: 12,
                flg1: Flg1(0b0000_0000),
                oldp: 13,
                newp: 14,
                upip: Some(IpAddr::V4([1, 2, 3, 4])),
                uppt: Some(15),
                uphp: Some(16),
            },
        };
        let mut buf = Vec::new();
        atom::serializer::to_writer(&mut buf, &before).unwrap();
        let atom: Bcst = atom::deserializer::from_reader(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(before, atom);
    }
}
