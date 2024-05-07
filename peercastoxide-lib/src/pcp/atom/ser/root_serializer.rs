use std::io::Write;

use serde::{Serialize, Serializer};

use crate::{common_unsupported_serializes, pcp::atom::ser::branch_serializer::BranchSerializer};

use super::{
    helpers::UnreachableSerializer, serialize_parent_struct::SerializeParentStruct,
    AtomSerializeError,
};

#[derive(derive_new::new)]
pub struct RootSerializer<W: Write> {
    writer: W,
    children_count: usize,
}

impl<W: Write> Serializer for RootSerializer<W> {
    type Ok = ();
    type Error = AtomSerializeError;
    type SerializeSeq = UnreachableSerializer<Self::Ok>;
    type SerializeTuple = UnreachableSerializer<Self::Ok>;
    type SerializeTupleStruct = UnreachableSerializer<Self::Ok>;
    type SerializeTupleVariant = UnreachableSerializer<Self::Ok>;
    type SerializeMap = UnreachableSerializer<Self::Ok>;
    type SerializeStruct = SerializeParentStruct<W>;
    type SerializeStructVariant = UnreachableSerializer<Self::Ok>;

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        debug_assert!(name.is_ascii());
        let mut identifier = [0u8; 4];
        identifier.copy_from_slice(format!("{:\0<4}", name).as_bytes());
        BranchSerializer::new(self.writer, &identifier, self.children_count)
            .serialize_newtype_struct(name, value)
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        debug_assert!(name.is_ascii());
        let mut identifier = [0u8; 4];
        identifier.copy_from_slice(format!("{:\0<4}", name).as_bytes());
        BranchSerializer::new(self.writer, &identifier, self.children_count)
            .serialize_struct(name, len)
    }

    // ----

    // unsupported structures

    common_unsupported_serializes! {}

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(AtomSerializeError::unsupported_structure("u8"))
    }
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
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(AtomSerializeError::unsupported_structure("seq"))
    }
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(AtomSerializeError::unsupported_structure("tuple"))
    }
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(AtomSerializeError::unsupported_structure("map"))
    }
}
