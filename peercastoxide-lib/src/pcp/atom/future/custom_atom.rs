use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    net::{Ipv4Addr, Ipv6Addr},
};

use std::ffi::{CString, NulError};

use crate::pcp::atom::to_string_without_zero_padding;

use super::{child::AtomChild, parent::AtomParent};

#[derive(Debug, Eq, PartialEq)]
pub struct Identifier(pub Cow<'static, [u8; 4]>);

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
pub enum CustomAtom {
    Parent(AtomParent),
    Child(AtomChild),
}

impl CustomAtom {
    pub fn parent(identifier: impl Into<Identifier>, children: Vec<CustomAtom>) -> Self {
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
            CustomAtom::Parent(parent) => parent.identifier(),
            CustomAtom::Child(child) => child.identifier(),
        }
    }

    pub fn to_identifier_string(&self) -> String {
        let identifier = match self {
            CustomAtom::Parent(parent) => parent.identifier(),
            CustomAtom::Child(child) => child.identifier(),
        };
        to_string_without_zero_padding(identifier)
    }
}

impl Display for CustomAtom {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parent(parent) => Display::fmt(parent, f),
            Self::Child(child) => Display::fmt(child, f),
        }
    }
}
