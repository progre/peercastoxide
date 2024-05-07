use std::io::{Cursor, Read};

use serde::{de::Visitor, Deserializer};

use crate::{common_unsupported_deserializes, pcp::atom::de::AtomDeserializeError};

pub struct ByteBufDeserializer {
    buf: Cursor<Vec<u8>>,
}

impl ByteBufDeserializer {
    pub fn new(buf: Vec<u8>) -> Self {
        Self {
            buf: Cursor::new(buf),
        }
    }

    pub fn has_remaining(&self) -> bool {
        (self.buf.position() as usize) < self.buf.get_ref().len()
    }
}

impl<'de> Deserializer<'de> for &mut ByteBufDeserializer {
    type Error = AtomDeserializeError;

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let mut buf = [0; 1];
        self.buf
            .read_exact(&mut buf)
            .map_err(AtomDeserializeError::Io)?;
        visitor.visit_u8(buf[0])
    }

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // maybe enum
        self.deserialize_u8(visitor)
    }

    // ----

    // unsupported structures

    common_unsupported_deserializes! {}

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
    fn deserialize_tuple<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        Err(AtomDeserializeError::unsupported_structure("tuple"))
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
