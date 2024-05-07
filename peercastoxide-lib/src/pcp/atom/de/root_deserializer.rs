use std::io::Read;

use anyhow::anyhow;
use serde::{de::Visitor, Deserializer};

use crate::{common_unsupported_deserializes, pcp::atom::unknown::Identifier};

use super::{
    atom_buf_reader::AtomBufReader, branch_deserializer::BranchDeserializer,
    children_map_access::ChildrenMapAccess, AtomDeserializeError,
};

pub struct RootDeserializer<R: Read> {
    reader: AtomBufReader<R>,
}

impl<R: Read> RootDeserializer<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader: AtomBufReader::new(reader),
        }
    }

    fn read_expected_identifier(&mut self, expected: &str) -> Result<(), AtomDeserializeError> {
        let identifier = Identifier::from(self.reader.read_identifier()?).to_string();
        if identifier != expected {
            let msg = anyhow!("identifier is expected {} but got {}", expected, identifier);
            return Err(AtomDeserializeError::Mismatch(msg));
        }
        Ok(())
    }
}

impl<'de, R: Read> Deserializer<'de> for RootDeserializer<R> {
    type Error = AtomDeserializeError;

    fn deserialize_newtype_struct<V>(
        mut self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.read_expected_identifier(name)?;
        visitor.visit_newtype_struct(&mut BranchDeserializer::new(&mut self.reader, vec![], 1))
    }

    fn deserialize_struct<V>(
        mut self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.read_expected_identifier(name)?;
        let count = self.reader.read_children_count()?;
        visitor.visit_map(ChildrenMapAccess::new(&mut self.reader, fields, count))
    }

    // ----

    // unsupported structures

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
    fn deserialize_seq<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(AtomDeserializeError::unsupported_structure("seq"))
    }
    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(AtomDeserializeError::unsupported_structure("tuple"))
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
