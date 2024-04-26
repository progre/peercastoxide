mod child;
mod parent;
pub mod well_known_atoms;
pub mod well_known_identifiers;
pub mod well_known_protocols;

use std::borrow::Cow;
use std::ffi::{CString, NulError};
use std::fmt::{Display, Formatter};
use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::Result;

pub use self::child::AtomChild;
pub use self::parent::AtomParent;
use self::well_known_identifiers::*;

fn to_string_without_zero_padding(string: &[u8]) -> String {
    String::from_utf8_lossy(string)
        .trim_end_matches('\0')
        .to_string()
}

#[derive(Debug, Eq, PartialEq)]
pub struct Identifier(Cow<'static, [u8; 4]>);

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

#[derive(Debug, Eq, PartialEq)]
pub enum Atom {
    Parent(AtomParent),
    Child(AtomChild),
}

impl Atom {
    pub fn parent(identifier: impl Into<Identifier>, children: Vec<Atom>) -> Self {
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

    pub fn identifier(&self) -> &[u8; 4] {
        match self {
            Atom::Parent(parent) => parent.identifier(),
            Atom::Child(child) => child.identifier(),
        }
    }

    pub fn to_identifier_string(&self) -> String {
        let identifier = match self {
            Atom::Parent(parent) => parent.identifier(),
            Atom::Child(child) => child.identifier(),
        };
        to_string_without_zero_padding(identifier)
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parent(parent) => parent.fmt(f),
            Self::Child(child) => child.fmt(f),
        }
    }
}
