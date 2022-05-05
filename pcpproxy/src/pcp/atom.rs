use crate::pcp::atom_identifier::*;
use derive_new::new;
use std::{borrow::Cow, io::Write};
use std::{fmt, net::Ipv4Addr};

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

#[derive(new)]
pub struct AtomParent {
    identifier: Cow<'static, [u8; 4]>,
    count: i32,
}

impl AtomParent {
    pub fn identifier(&self) -> &[u8; 4] {
        self.identifier.as_ref()
    }

    pub fn count(&self) -> i32 {
        self.count
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
        (&mut num[0..2]).write(&self.data).unwrap();
        u16::from_le_bytes(num)
    }

    pub fn to_cow_str<'a>(&'a self) -> Cow<'a, str> {
        String::from_utf8_lossy(&self.data)
    }

    pub fn to_ipv4(&self) -> Ipv4Addr {
        Ipv4Addr::new(self.data[3], self.data[2], self.data[1], self.data[0])
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

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Atom::Parent(parent) => write!(
                f,
                "{}:    # children: {}",
                to_string(parent.identifier.as_ref()),
                parent.count
            ),
            Atom::Child(child) => {
                let id_str = to_string(child.identifier.as_ref());
                let content_str = match child.identifier.as_ref() {
                    PORT | UPPT | VEXP | VEXN if child.data().len() == 2 => {
                        child.to_u16().to_string()
                    }
                    NEWP | NUML | NUMR | OLDP | QUIT | UPHP | UPPT | UPTM | VER | VERS | VEVP
                    | VRVP
                        if child.data().len() == 4 =>
                    {
                        let mut num = [0u8; 4];
                        (&mut num[0..4]).write(child.data()).unwrap();
                        u32::from_le_bytes(num).to_string()
                    }
                    IP | RIP | UPIP if child.data().len() == 4 => child.to_ipv4().to_string(),
                    CID | FROM | ID | SID if child.data().len() == 16 => child
                        .data()
                        .iter()
                        .map(|&x| format!("{:x}", x))
                        .collect::<Vec<_>>()
                        .join(""),
                    AGNT | ALBM | CMNT | CREA | DESC | GNRE | NAME | STYP | SEXT | TITL | TYPE
                    | URL => child.to_cow_str().into(),
                    DATA => format!("({} bytes)", child.data().len()),
                    FLG1 if child.data().len() == 1 => from_flg1_to_string(child.data()[0]),
                    _ => format!("{:?}", child.data()),
                };
                write!(f, "{}: {}", id_str, content_str)
            }
        }
    }
}
