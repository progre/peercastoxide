use std::net::Ipv4Addr;
use std::{borrow::Cow, io::Write};

use derive_new::new;
use getset::Getters;
use serde::ser::SerializeMap;
use serde::Serialize;

use super::atom_identifier::*;

fn to_string(identifier: &[u8; 4]) -> String {
    identifier
        .iter()
        .map(|&x| (x as char).to_string())
        .collect::<Vec<_>>()
        .join("")
}

fn from_flg1_to_string(data: u8) -> String {
    let tracker = data & 1 << 0 != 0;
    let relay = data & 1 << 1 != 0;
    let direct = data & 1 << 2 != 0;
    let push = data & 1 << 3 != 0;
    let recv = data & 1 << 4 != 0;
    let cin = data & 1 << 5 != 0;
    let private = data & 1 << 6 != 0;
    let unused = data & 1 << 7 != 0;
    format!(
        "{}tracker{}{}relay{}{}direct{}{}push{}{}recv{}{}cin{}{}private{}{}?{}",
        if tracker { '[' } else { ' ' },
        if tracker { ']' } else { ' ' },
        if relay { '[' } else { ' ' },
        if relay { ']' } else { ' ' },
        if direct { '[' } else { ' ' },
        if direct { ']' } else { ' ' },
        if push { '[' } else { ' ' },
        if push { ']' } else { ' ' },
        if recv { '[' } else { ' ' },
        if recv { ']' } else { ' ' },
        if cin { '[' } else { ' ' },
        if cin { ']' } else { ' ' },
        if private { '[' } else { ' ' },
        if private { ']' } else { ' ' },
        if unused { '[' } else { ' ' },
        if unused { ']' } else { ' ' },
    )
}

#[derive(Getters)]
pub struct AtomParent {
    #[getset(get = "pub")]
    identifier: Cow<'static, [u8; 4]>,
    #[getset(get = "pub")]
    children: Vec<Atom>,
}

impl AtomParent {
    pub fn new(identifier: Cow<'static, [u8; 4]>, children: Vec<Atom>) -> Self {
        Self {
            identifier,
            children,
        }
    }
}

impl Serialize for AtomParent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry(
            "identifier",
            &String::from_utf8_lossy(self.identifier().as_ref()),
        )?;
        map.serialize_entry("children", self.children())?;
        map.end()
    }
}

#[derive(new)]
pub struct AtomChild {
    identifier: Cow<'static, [u8; 4]>,
    data: Vec<u8>,
}

impl AtomChild {
    pub fn u16(identifier: Cow<'static, [u8; 4]>, data: u16) -> Self {
        Self::new(identifier, data.to_le_bytes().to_vec())
    }

    pub fn ipv4(identifier: Cow<'static, [u8; 4]>, data: Ipv4Addr) -> Self {
        let mut octets = data.octets();
        octets.reverse();
        Self::new(identifier, octets.to_vec())
    }

    pub fn identifier(&self) -> &[u8; 4] {
        self.identifier.as_ref()
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn to_u16(&self) -> u16 {
        let mut num = [0u8; 2];
        (&mut num[0..2]).write_all(&self.data).unwrap();
        u16::from_le_bytes(num)
    }

    pub fn to_ipv4(&self) -> Ipv4Addr {
        Ipv4Addr::new(self.data[3], self.data[2], self.data[1], self.data[0])
    }
}

impl Serialize for AtomChild {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        let id_str = String::from_utf8_lossy(self.identifier.as_ref());
        map.serialize_entry("identifier", &id_str)?;

        match self.identifier.as_ref() {
            PORT | UPPT | VEXP | VEXN if self.data().len() == 2 => {
                map.serialize_entry("payload", &self.to_u16())?;
            }
            NEWP | NUML | NUMR | OLDP | QUIT | UPHP | UPPT | UPTM | VER | VERS | VEVP | VRVP
                if self.data().len() == 4 =>
            {
                let mut num = [0u8; 4];
                (&mut num[0..4]).write_all(self.data()).unwrap();
                map.serialize_entry("payload", &u32::from_le_bytes(num))?;
            }
            IP | RIP | UPIP if self.data().len() == 4 => {
                map.serialize_entry("payload", &self.to_ipv4().to_string())?;
            }
            CID | FROM | ID | SID if self.data().len() == 16 => {
                let value = self
                    .data()
                    .iter()
                    .map(|&x| format!("{:x}", x))
                    .collect::<Vec<_>>()
                    .join("");
                map.serialize_entry("payload", &value)?;
            }
            AGNT | ALBM | CMNT | CREA | DESC | GNRE | NAME | STYP | SEXT | TITL | TYPE | URL => {
                map.serialize_entry("payload", &String::from_utf8_lossy(self.data()))?;
            }
            DATA => {
                map.serialize_entry("payload", &format!("({} bytes)", self.data().len()))?;
            }
            FLG1 if self.data().len() == 1 => {
                map.serialize_entry("payload", &from_flg1_to_string(self.data()[0]))?;
            }
            _ => {
                map.serialize_entry("payload", self.data())?;
            }
        };
        map.end()
    }
}

pub enum Atom {
    Parent(AtomParent),
    Child(AtomChild),
}

impl Atom {
    pub fn identifier(&self) -> &[u8; 4] {
        match self {
            Atom::Parent(parent) => parent.identifier(),
            Atom::Child(child) => child.identifier(),
        }
    }
}

impl Serialize for Atom {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Atom::Parent(parent) => parent.serialize(serializer),
            Atom::Child(child) => child.serialize(serializer),
        }
    }
}
