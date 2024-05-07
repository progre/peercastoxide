use std::io::Write;

use serde::{Serialize, Serializer};

use crate::common_unsupported_serializes;

use super::{helpers::UnreachableSerializer, AtomSerializeError};

#[derive(derive_new::new)]
pub struct DataSerializer<W: Write> {
    writer: W,
}

impl<W: Write> Serializer for &mut DataSerializer<W> {
    type Ok = ();
    type Error = AtomSerializeError;
    type SerializeSeq = UnreachableSerializer<Self::Ok>;
    type SerializeTuple = UnreachableSerializer<Self::Ok>;
    type SerializeTupleStruct = UnreachableSerializer<Self::Ok>;
    type SerializeTupleVariant = UnreachableSerializer<Self::Ok>;
    type SerializeMap = UnreachableSerializer<Self::Ok>;
    type SerializeStruct = UnreachableSerializer<Self::Ok>;
    type SerializeStructVariant = UnreachableSerializer<Self::Ok>;

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(&v.to_le_bytes())
            .map_err(AtomSerializeError::Io)?;
        Ok(())
    }

    // ----

    // unsupported structures

    common_unsupported_serializes! {}

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(AtomSerializeError::unsupported_structure("u16"))
    }
    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(AtomSerializeError::unsupported_structure("u32"))
    }
    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        Err(AtomSerializeError::unsupported_structure("str"))
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(AtomSerializeError::unsupported_structure("none"))
    }
    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(AtomSerializeError::unsupported_structure("some"))
    }
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(AtomSerializeError::unsupported_structure("newtype struct"))
    }
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(AtomSerializeError::unsupported_structure("seq"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(AtomSerializeError::unsupported_structure("tuple"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(AtomSerializeError::unsupported_structure("map"))
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(AtomSerializeError::unsupported_structure("struct"))
    }
}
