use anyhow::Result;
use serde::{
    ser::{SerializeSeq, SerializeStruct, SerializeTuple},
    Serialize, Serializer,
};

use crate::{
    common_unsupported_serializes,
    pcp::atom::ser::{helpers::UnreachableSerializer, AtomSerializeError},
};

use super::branch_serializer::BranchSerializer;

#[derive(Default, getset::CopyGetters)]
pub struct RootSerializer {
    #[get_copy = "pub"]
    result: usize,
}

impl RootSerializer {
    pub fn new() -> Self {
        Self { result: 0 }
    }
}

impl Serializer for &mut RootSerializer {
    type Ok = ();
    type Error = AtomSerializeError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = UnreachableSerializer<Self::Ok>;
    type SerializeTupleVariant = UnreachableSerializer<Self::Ok>;
    type SerializeMap = UnreachableSerializer<Self::Ok>;
    type SerializeStruct = Self;
    type SerializeStructVariant = UnreachableSerializer<Self::Ok>;

    common_unsupported_serializes! {}

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        BranchSerializer::new(None).serialize_u8(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        BranchSerializer::new(None).serialize_u16(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        BranchSerializer::new(None).serialize_u32(v)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        BranchSerializer::new(None).serialize_str(v)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        BranchSerializer::new(None).serialize_none()
    }

    fn serialize_some<T>(self, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        BranchSerializer::new(None).serialize_some(v)
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        BranchSerializer::new(None).serialize_newtype_struct(name, value)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        self.result += 1;
        Ok(self)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.result += 1;
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
        Ok(self)
    }
}

impl SerializeSeq for &mut RootSerializer {
    type Ok = ();
    type Error = AtomSerializeError;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeTuple for &mut RootSerializer {
    type Ok = ();
    type Error = AtomSerializeError;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl SerializeStruct for &mut RootSerializer {
    type Ok = ();
    type Error = AtomSerializeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        // 5 文字以上は grouped atoms なので 1 要素以上の数になる
        let atoms = (key.len() + 3) / 4;
        let mut child = BranchSerializer::new(if atoms == 1 { None } else { Some(atoms) });
        value.serialize(&mut child)?;
        self.result += child.result();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
