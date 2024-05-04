use std::fmt::{Display, Formatter};

use derive_new::new;
use getset::Getters;

use crate::pcp::atom::to_string_without_zero_padding;

use super::{Identifier, UnknownAtom};

#[derive(Debug, Eq, Getters, PartialEq, new)]
pub struct AtomParent {
    identifier: Identifier,
    #[getset(get = "pub")]
    children: Vec<UnknownAtom>,
}

impl AtomParent {
    pub fn identifier(&self) -> &[u8; 4] {
        self.identifier.0.as_ref()
    }

    pub fn children_mut(&mut self) -> &mut Vec<UnknownAtom> {
        &mut self.children
    }
}

impl Display for AtomParent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let is_pretty = f.alternate();
        f.write_str(&format!(
            "{:?} ",
            to_string_without_zero_padding(self.identifier())
        ))?;
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
        Ok(())
    }
}
