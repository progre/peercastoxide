use std::{ffi::CStr, io::Read};

use anyhow::anyhow;
use serde::{de::Visitor, Deserializer};

use crate::{common_unsupported_deserializes, pcp::atom::unknown::Identifier};

use super::{
    atom_buf_reader::AtomBufReader, children_map_access::ChildrenMapAccess,
    grouped_atoms::seq_access::GroupedAtomsSeqAccess, vec_data_seq_access::VecDataSeqAccess,
    AtomDeserializeError,
};

fn raw_grouped_atoms_to_string(grouped_atoms: &[[u8; 4]]) -> String {
    grouped_atoms
        .iter()
        .flatten()
        .map(|&x| x as char)
        .collect::<String>()
}

#[derive(getset::CopyGetters, getset::MutGetters)]
pub struct BranchDeserializer<'a, R: Read> {
    #[get_mut = "pub"]
    reader: &'a mut AtomBufReader<R>,
    grouped_atoms_list: Vec<Vec<[u8; 4]>>,
    found_grouped_atoms_idx: Option<usize>,
    #[getset(get_copy = "pub")]
    remaining: u32,
}

impl<'a, R: Read> BranchDeserializer<'a, R> {
    pub fn new(
        reader: &'a mut AtomBufReader<R>,
        grouped_atoms_list: Vec<Vec<[u8; 4]>>,
        remaining: u32,
    ) -> Self {
        Self {
            reader,
            grouped_atoms_list,
            found_grouped_atoms_idx: None,
            remaining,
        }
    }
}

impl<'a, 'de, R: Read> Deserializer<'de> for &mut BranchDeserializer<'a, R> {
    type Error = AtomDeserializeError;

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let raw_identifier = self.reader.peek_identifier()?;
        if let Some((idx, grouped_atoms)) = self
            .grouped_atoms_list
            .iter()
            .enumerate()
            .find(|(_, x)| x[0] == raw_identifier)
        {
            let grouped_atoms = raw_grouped_atoms_to_string(grouped_atoms);
            self.found_grouped_atoms_idx = Some(idx);
            tracing::trace! { %grouped_atoms, "deserialize_identifier" }
            return visitor.visit_string(grouped_atoms);
        }
        let identifier = Identifier::from(self.reader.read_identifier()?).to_string();
        visitor.visit_string(identifier)
    }

    // ----

    common_unsupported_deserializes! {}

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // maybe enum
        let buf = self.reader.read_data_size_and_byte_buf()?;
        self.remaining -= 1;
        visitor.visit_seq(VecDataSeqAccess::new(buf))
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let size = self.reader.read_data_size()?;
        if size != 1 {
            let err = anyhow!("data size is expected 1 but got {}", size);
            return Err(AtomDeserializeError::Mismatch(err));
        }
        self.remaining -= 1;
        visitor.visit_u8(self.reader.read_u8()?)
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let size = self.reader.read_data_size()?;
        if size != 2 {
            let err = anyhow!("data size is expected 2 but got {}", size);
            return Err(AtomDeserializeError::Mismatch(err));
        }
        self.remaining -= 1;
        visitor.visit_u16(self.reader.read_u16()?)
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let size = self.reader.read_data_size()?;
        if size != 4 {
            let err = anyhow!("data size is expected 4 but got {}", size);
            return Err(AtomDeserializeError::Mismatch(err));
        }
        self.remaining -= 1;
        visitor.visit_u32(self.reader.read_u32()?)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let buf = self.reader.read_data_size_and_byte_buf()?;
        self.remaining -= 1;
        let cstr = CStr::from_bytes_with_nul(&buf)
            .map_err(|e| AtomDeserializeError::Mismatch(e.into()))?;
        visitor.visit_string(cstr.to_string_lossy().into_owned())
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(AtomDeserializeError::unsupported_structure(
            "newtype struct",
        ))
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let Some(idx) = self.found_grouped_atoms_idx.take() else {
            let data = self.reader.read_data_size_and_byte_buf()?;
            self.remaining -= 1;
            return visitor.visit_seq(VecDataSeqAccess::new(data));
        };
        // grouped_atoms
        let grouped_atoms = &self.grouped_atoms_list[idx];
        let seq = GroupedAtomsSeqAccess::new(self.reader, grouped_atoms, &mut self.remaining);
        visitor.visit_seq(seq)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let size = self.reader.read_data_size()?;
        if size as usize != len {
            let err = anyhow!("data size is expected {} but got {}", len, size);
            return Err(AtomDeserializeError::Mismatch(err));
        }
        let mut buf = vec![0u8; size as usize];
        self.reader.read_exact(&mut buf)?;
        self.remaining -= 1;
        visitor.visit_seq(VecDataSeqAccess::new(buf))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let count = self.reader.read_children_count()?;
        self.remaining -= 1;
        visitor.visit_map(ChildrenMapAccess::new(self.reader, fields, count))
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(AtomDeserializeError::unsupported_structure("enum"))
    }
}
