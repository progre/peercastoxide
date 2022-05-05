use crate::pcp::atom_identifier::*;
use derive_new::new;
use std::{borrow::Cow, io::Write};
use std::{fmt, net::Ipv4Addr};

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
    count: i32,
}

impl AtomParent {
    pub fn count(&self) -> i32 {
        self.count
    }
}

#[derive(new)]
pub struct AtomChild {
    data: Vec<u8>,
}

impl AtomChild {
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn to_ipv4(&self) -> Ipv4Addr {
        Ipv4Addr::new(self.data[3], self.data[2], self.data[1], self.data[0])
    }
}

pub struct AtomIdentifier(Cow<'static, [u8; 4]>);

impl AtomIdentifier {
    pub fn borrowed(id: &'static [u8; 4]) -> Self {
        Self(Cow::Borrowed(id))
    }

    pub fn owned(id: [u8; 4]) -> Self {
        Self(Cow::Owned(id))
    }
}

pub enum AtomContent {
    Parent(AtomParent),
    Child(AtomChild),
}

pub struct Atom {
    identifier: AtomIdentifier,
    content: AtomContent,
}

impl Atom {
    pub fn parent(identifier: AtomIdentifier, count: i32) -> Self {
        Self {
            identifier,
            content: AtomContent::Parent(AtomParent { count }),
        }
    }

    pub fn child(identifier: AtomIdentifier, data: Vec<u8>) -> Self {
        Self {
            identifier,
            content: AtomContent::Child(AtomChild { data }),
        }
    }

    pub fn ipv4(identifier: impl Into<Cow<'static, [u8; 4]>>, ipv4: Ipv4Addr) -> Self {
        let mut octets = ipv4.octets();
        octets.reverse();
        Self {
            identifier: identifier.into(),
            content: AtomContent::Child(AtomChild {
                data: octets.to_vec(),
            }),
        }
    }

    pub fn identifier(&self) -> &[u8; 4] {
        &self.identifier
    }

    pub fn content(&self) -> &AtomContent {
        &self.content
    }

    pub fn to_identifier_string(&self) -> String {
        self.identifier
            .iter()
            .map(|&x| (x as char).to_string())
            .collect::<Vec<_>>()
            .join("")
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.content {
            AtomContent::Parent(AtomParent { count }) => write!(
                f,
                "{}:    # children: {}",
                self.to_identifier_string(),
                count
            ),
            AtomContent::Child(child) => {
                let id_str = self.to_identifier_string();
                let content_str = match self.identifier.as_ref() {
                    PORT | UPPT | VEXP | VEXN if child.data().len() == 2 => {
                        let mut num = [0u8; 2];
                        (&mut num[0..2]).write(child.data()).unwrap();
                        u16::from_le_bytes(num).to_string()
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
                    AGNT => child
                        .data()
                        .iter()
                        .map(|&x| (x as char).to_string())
                        .collect::<Vec<_>>()
                        .join(""),
                    DATA => format!("({} bytes)", child.data().len()),
                    FLG1 if child.data().len() == 1 => from_flg1_to_string(child.data()[0]),
                    _ => format!("{:?}", child.data()),
                };
                write!(f, "{}: {}", id_str, content_str)
            }
        }
    }
}
