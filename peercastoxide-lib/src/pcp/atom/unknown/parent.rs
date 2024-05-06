use std::fmt::{Display, Formatter};

use derive_new::new;
use getset::Getters;

use super::{Identifier, UnknownAtom};

#[derive(Debug, Eq, Getters, PartialEq, new)]
pub struct AtomParent {
    #[get = "pub"]
    identifier: Identifier,
    #[get = "pub"]
    children: Vec<UnknownAtom>,
}

impl AtomParent {
    pub fn children_mut(&mut self) -> &mut Vec<UnknownAtom> {
        &mut self.children
    }
}

impl Display for AtomParent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let identifier = self.identifier().to_string();
        if identifier.chars().all(|x| !x.is_ascii_control()) {
            write!(f, "{}(", identifier)?;
        } else {
            write!(f, "{:?}(", identifier)?;
        };
        let is_pretty = f.alternate();
        let mut s = f.debug_list();
        for child in &self.children {
            match child {
                UnknownAtom::Parent(parent) => {
                    if is_pretty {
                        s.entry(&format_args!("{:#}", parent));
                    } else {
                        s.entry(&format_args!("{}", parent));
                    }
                }
                UnknownAtom::Child(child) => {
                    if is_pretty {
                        s.entry(&format_args!("{:#}", child));
                    } else {
                        s.entry(&format_args!("{}", child));
                    }
                }
            }
        }
        s.finish()?;
        write!(f, ")")
    }
}
