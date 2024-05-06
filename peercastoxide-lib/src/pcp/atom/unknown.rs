mod child;
mod parent;
mod well_known_identifiers;

use std::{
    borrow::Cow,
    ffi::{CString, NulError},
    fmt::{Display, Formatter},
    io::Cursor,
    net::{Ipv4Addr, Ipv6Addr},
};

use anyhow::anyhow;

use crate::pcp::atom;

pub use self::{child::AtomChild, parent::AtomParent};

use super::{
    atom_stream::{AtomStreamReader, AtomStreamWriter},
    de::AtomDeserializeError,
    ser::AtomSerializeError,
};

pub fn from_unknown<'de, T: serde::Deserialize<'de>>(
    unknown_atom: UnknownAtom,
) -> Result<T, super::de::AtomDeserializeError> {
    let mut vec = Vec::new();
    tokio::task::block_in_place(|| {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(AtomStreamWriter::new(&mut vec).write_unknown_atom(&unknown_atom))
            .map_err(AtomDeserializeError::UnsupportedStructure)
    })?;
    atom::from_reader(&mut Cursor::new(vec))
}

pub fn to_unknown<T: serde::Serialize>(
    value: &T,
) -> Result<UnknownAtom, super::ser::AtomSerializeError> {
    let mut vec = Vec::new();
    crate::pcp::atom::to_writer(&mut vec, value)?;
    let atom = tokio::task::block_in_place(|| {
        tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(AtomStreamReader::new(&mut Cursor::new(vec)).read_unknown_atom())
            .map_err(AtomSerializeError::UnsupportedStructure)
    })?;
    Ok(atom)
}

#[derive(Debug, Eq, PartialEq)]
pub struct Identifier(pub Cow<'static, [u8; 4]>);

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let buf = self
            .0
            .into_iter()
            .take_while(|&x| x != b'\0')
            .map(|x| x as char)
            .collect::<String>();
        write!(f, "{}", buf)
    }
}

impl From<&'static [u8; 4]> for Identifier {
    fn from(data: &'static [u8; 4]) -> Self {
        Self(Cow::Borrowed(data))
    }
}

impl From<[u8; 4]> for Identifier {
    fn from(data: [u8; 4]) -> Self {
        Self(Cow::Owned(data))
    }
}

impl TryFrom<&str> for Identifier {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let buf = value.as_bytes();
        if buf.len() > 4 {
            return Err(anyhow!("Identifier must be 4 bytes or less"));
        }
        Ok(Self(Cow::Owned([
            buf.first().copied().unwrap_or(b'\0'),
            buf.get(1).copied().unwrap_or(b'\0'),
            buf.get(2).copied().unwrap_or(b'\0'),
            buf.get(3).copied().unwrap_or(b'\0'),
        ])))
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum UnknownAtom {
    Parent(AtomParent),
    Child(AtomChild),
}

impl UnknownAtom {
    pub fn parent(identifier: impl Into<Identifier>, children: Vec<UnknownAtom>) -> Self {
        Self::Parent(AtomParent::new(identifier.into(), children))
    }

    pub fn child(identifier: impl Into<Identifier>, data: Vec<u8>) -> Self {
        Self::Child(AtomChild::new(identifier.into(), data))
    }

    pub fn ipv4(identifier: impl Into<Identifier>, data: &Ipv4Addr) -> Self {
        let mut octets = data.octets();
        octets.reverse();
        Self::Child(AtomChild::new(identifier.into(), octets.to_vec()))
    }

    pub fn ipv6(identifier: impl Into<Identifier>, data: &Ipv6Addr) -> Self {
        let mut octets = data.octets();
        octets.reverse();
        Self::Child(AtomChild::new(identifier.into(), octets.to_vec()))
    }

    pub fn str(identifier: impl Into<Identifier>, data: &str) -> Result<Self, NulError> {
        Ok(Self::Child(AtomChild::new(
            identifier.into(),
            CString::new(data)?.as_bytes_with_nul().to_vec(),
        )))
    }

    pub fn u16(identifier: impl Into<Identifier>, data: u16) -> Self {
        Self::Child(AtomChild::new(
            identifier.into(),
            data.to_le_bytes().to_vec(),
        ))
    }

    pub fn u32(identifier: impl Into<Identifier>, data: u32) -> Self {
        Self::Child(AtomChild::new(
            identifier.into(),
            data.to_le_bytes().to_vec(),
        ))
    }

    pub fn identifier(&self) -> &Identifier {
        match self {
            UnknownAtom::Parent(parent) => parent.identifier(),
            UnknownAtom::Child(child) => child.identifier(),
        }
    }

    pub fn to_identifier_string(&self) -> String {
        let identifier = match self {
            UnknownAtom::Parent(parent) => parent.identifier(),
            UnknownAtom::Child(child) => child.identifier(),
        };
        String::from_utf8_lossy(identifier.0.as_ref())
            .trim_end_matches('\0')
            .to_string()
    }
}

impl Display for UnknownAtom {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parent(parent) => Display::fmt(parent, f),
            Self::Child(child) => Display::fmt(child, f),
        }
    }
}
