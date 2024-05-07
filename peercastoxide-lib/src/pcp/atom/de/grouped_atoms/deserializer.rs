use std::io::Read;

use serde::{de::Visitor, Deserializer};

use crate::{
    common_unsupported_deserializes,
    pcp::atom::de::{atom_buf_reader::AtomBufReader, AtomDeserializeError},
};

use super::each_atoms_seq_access::GroupedAtomsEachAtomSeqAccess;

#[derive(getset::Getters, getset::MutGetters)]
pub struct GroupedAtomsDeserializer<'a, R: Read> {
    #[get_mut = "pub"]
    reader: &'a mut AtomBufReader<R>,
    grouped_atoms: &'a [[u8; 4]],
    remaining: &'a mut u32,
}

impl<'a, R: Read> GroupedAtomsDeserializer<'a, R> {
    pub fn new(
        reader: &'a mut AtomBufReader<R>,
        grouped_atoms: &'a [[u8; 4]],
        remaining: &'a mut u32,
    ) -> Self {
        Self {
            reader,
            grouped_atoms,
            remaining,
        }
    }

    pub fn has_remaining(&self) -> bool {
        *self.remaining > 0
    }
}

impl<'a, 'de, R: Read> Deserializer<'de> for &mut GroupedAtomsDeserializer<'a, R> {
    type Error = AtomDeserializeError;

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let seq =
            GroupedAtomsEachAtomSeqAccess::new(self.reader, self.grouped_atoms, self.remaining);
        visitor.visit_seq(seq)
    }

    common_unsupported_deserializes! {}

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(AtomDeserializeError::unsupported_structure("any"))
    }
    fn deserialize_u8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(AtomDeserializeError::unsupported_structure("u8"))
    }
    fn deserialize_u16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(AtomDeserializeError::unsupported_structure("u16"))
    }
    fn deserialize_u32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(AtomDeserializeError::unsupported_structure("u32"))
    }
    fn deserialize_string<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(AtomDeserializeError::unsupported_structure("string"))
    }
    fn deserialize_option<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(AtomDeserializeError::unsupported_structure("option"))
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
    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(AtomDeserializeError::unsupported_structure("seq"))
    }
    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(AtomDeserializeError::unsupported_structure("struct"))
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
    fn deserialize_identifier<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(AtomDeserializeError::unsupported_structure("identifier"))
    }
}
