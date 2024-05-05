use std::io::Write;

use anyhow::Result;
use serde::{
    ser::{SerializeTuple, SerializeTupleStruct},
    Serialize, Serializer,
};

use crate::{
    common_unsupported_serializes,
    pcp::atom::ser::{
        branch_serializer::BranchSerializer, count_children::count_children,
        helpers::UnreachableSerializer, AtomSerializeError,
    },
};

pub struct GroupedAtomsSerializer<'a, W: Write> {
    writer: W,
    grouped_atoms: &'a [[u8; 4]],
    idx: usize,
}

impl<'a, W: Write> GroupedAtomsSerializer<'a, W> {
    pub fn new(writer: W, grouped_atoms: &'a [[u8; 4]]) -> Self {
        Self {
            writer,
            grouped_atoms,
            idx: 0,
        }
    }
}

impl<'a, W: Write> Serializer for GroupedAtomsSerializer<'a, W> {
    type Ok = ();
    type Error = AtomSerializeError;
    type SerializeSeq = UnreachableSerializer<Self::Ok>;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = UnreachableSerializer<Self::Ok>;
    type SerializeMap = UnreachableSerializer<Self::Ok>;
    type SerializeStruct = UnreachableSerializer<Self::Ok>;
    type SerializeStructVariant = UnreachableSerializer<Self::Ok>;

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

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        if len != self.grouped_atoms.len() {
            let err = anyhow::anyhow!(
                "grouped atoms length expected {} but got {}",
                self.grouped_atoms.len(),
                len
            );
            return Err(AtomSerializeError::UnsupportedStructure(err));
        }
        Ok(self)
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

impl<'a, W: Write> SerializeTuple for GroupedAtomsSerializer<'a, W> {
    type Ok = ();
    type Error = AtomSerializeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let identifier = self.grouped_atoms[self.idx];
        self.idx += 1;
        value.serialize(BranchSerializer::new(
            &mut self.writer,
            &identifier,
            count_children(value)?,
        ))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeTupleStruct for GroupedAtomsSerializer<'a, W> {
    type Ok = ();
    type Error = AtomSerializeError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let identifier = self.grouped_atoms[self.idx];
        self.idx += 1;
        value.serialize(BranchSerializer::new(
            &mut self.writer,
            &identifier,
            count_children(value)?,
        ))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
