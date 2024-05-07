mod branch_serializer;
mod count_children;
mod data_serializer;
mod grouped_atoms;
pub mod helpers;
mod root_serializer;
mod serialize_parent_struct;

use std::{
    fmt::Display,
    io::{self, Write},
};

use anyhow::{anyhow, Result};
use serde::Serialize;

use self::{count_children::count_children, root_serializer::RootSerializer};

#[derive(Debug, thiserror::Error)]
pub enum AtomSerializeError {
    #[error("unsupported structure")]
    UnsupportedStructure(#[source] anyhow::Error),
    #[error("io error")]
    Io(#[source] io::Error),
    #[error("error")]
    Serde(#[source] anyhow::Error),
}

impl AtomSerializeError {
    pub fn unsupported_structure(structure: &'static str) -> Self {
        let err = anyhow!("{} is not supported", structure);
        AtomSerializeError::UnsupportedStructure(err)
    }
}

impl serde::ser::Error for AtomSerializeError {
    fn custom<T: Display>(msg: T) -> Self {
        AtomSerializeError::Serde(anyhow!(msg.to_string()))
    }
}

pub fn to_writer(writer: impl Write, value: impl Serialize) -> Result<(), AtomSerializeError> {
    value.serialize(RootSerializer::new(writer, count_children(&value)?))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::pcp::atom::{
        self,
        ser::AtomSerializeError,
        values::{AtomIpAddr, Flg1, Id, VExP},
        well_known_atoms::{Bcst, Chan, Helo, Host, Info, Oleh, Pcp, Trck},
    };

    #[test]
    fn test_unsupported_structure() {
        #[derive(serde::Serialize)]
        struct Unsupported;

        let Err(err) = atom::ser::to_writer(Vec::new(), &Unsupported) else {
            panic!()
        };
        assert!(matches!(err, AtomSerializeError::UnsupportedStructure(_)));
    }

    #[test]
    fn test_to_writer_pcp() {
        let before = Pcp(1);
        let mut buf = Vec::new();
        atom::ser::to_writer(&mut buf, &before).unwrap();
        let after: Pcp = atom::de::from_reader(&mut Cursor::new(&buf)).unwrap();
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
        atom::ser::to_writer(&mut buf, &before).unwrap();
        let after: Helo = atom::de::from_reader(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn test_to_writer_helo_full() {
        let before = Helo {
            sid: Id([1; 16]),
            agnt: Some("agent".into()),
            ver: Some(2),
            port: Some(3),
            ping: Some(4),
            bcid: Some(Id([5; 16])),
        };
        let mut buf = Vec::new();
        atom::ser::to_writer(&mut buf, &before).unwrap();
        let after: Helo = atom::de::from_reader(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn test_to_writer_oleh() {
        let before = Oleh {
            sid: Id([1; 16]),
            agnt: Some("agent".into()),
            rip: Some(AtomIpAddr::from([1, 2, 3, 4])),
            port: Some(5),
            ver: Some(7),
        };
        let mut buf = Vec::new();
        atom::ser::to_writer(&mut buf, &before).unwrap();
        let after: Oleh = atom::de::from_reader(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn test_to_writer_chan() {
        let before = Chan {
            id: Id([2; 16]),
            bcid: Id([3; 16]),
            info: Info {
                name: "name".into(),
                bitr: Some(20),
                gnre: "genre".into(),
                url: "url".into(),
                desc: "description".into(),
                cmnt: "comment".into(),
                r#type: Some("type".into()),
                styp: Some("subtype".into()),
                sext: Some("secondary_ext".into()),
            },
            trck: Trck {
                titl: "title".into(),
                crea: "creator".into(),
                url: "url".into(),
                albm: "album".into(),
                gnre: Some("genre".into()),
            },
        };
        let mut buf = Vec::new();
        atom::ser::to_writer(&mut buf, &before).unwrap();
        let after: Chan = atom::de::from_reader(&mut Cursor::new(&buf)).unwrap();
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
            cid: Some(Id([6; 16])),
            chan: Chan {
                id: Id([2; 16]),
                bcid: Id([3; 16]),
                info: Info {
                    name: "name".into(),
                    bitr: Some(20),
                    gnre: "genre".into(),
                    url: "url".into(),
                    desc: "description".into(),
                    cmnt: "comment".into(),
                    r#type: Some("type".into()),
                    styp: Some("subtype".into()),
                    sext: Some("secondary_ext".into()),
                },
                trck: Trck {
                    titl: "title".into(),
                    crea: "creator".into(),
                    url: "url".into(),
                    albm: "album".into(),
                    gnre: Some("genre".into()),
                },
            },
            host: Host {
                cid: Id([4; 16]),
                id: Id([5; 16]),
                ip_port: vec![
                    (AtomIpAddr::from([1, 2, 3, 4]), 5),
                    (
                        AtomIpAddr::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
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
                oldp: Some(13),
                newp: Some(14),
                upip: Some(AtomIpAddr::from([1, 2, 3, 4])),
                uppt: Some(15),
                uphp: Some(16),
            },
        };
        let mut buf = Vec::new();
        atom::ser::to_writer(&mut buf, &before).unwrap();
        let atom: Bcst = atom::de::from_reader(&mut Cursor::new(&buf)).unwrap();
        assert_eq!(before, atom);
    }
}
