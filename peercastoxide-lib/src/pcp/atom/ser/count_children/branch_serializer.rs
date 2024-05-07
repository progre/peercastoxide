use anyhow::Result;
use serde::{
    ser::{SerializeSeq, SerializeStruct, SerializeTuple},
    Serialize, Serializer,
};

use crate::{
    common_unsupported_serializes,
    pcp::atom::ser::{helpers::UnreachableSerializer, AtomSerializeError},
};

#[derive(getset::CopyGetters)]
pub struct BranchSerializer {
    grouped_atoms_size: Option<usize>,
    #[get_copy = "pub"]
    result: usize,
}

impl BranchSerializer {
    pub fn new(grouped_atoms_size: Option<usize>) -> Self {
        Self {
            grouped_atoms_size,
            result: 0,
        }
    }
}

impl Serializer for &mut BranchSerializer {
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

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        self.result += 1;
        Ok(())
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        self.result += 1;
        Ok(())
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        self.result += 1;
        Ok(())
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        self.result += 1;
        Ok(())
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T>(self, v: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let mut child = BranchSerializer::new(None);
        v.serialize(&mut child)?;
        self.result += child.result;
        Ok(())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.result += 1;
        Ok(())
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if let Some(grouped_atoms_size) = self.grouped_atoms_size {
            self.result += grouped_atoms_size * len.unwrap();
        } else {
            self.result += 1;
        }
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
        self.result += 1;
        Ok(self)
    }
}

impl SerializeSeq for &mut BranchSerializer {
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

impl SerializeTuple for &mut BranchSerializer {
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

impl SerializeStruct for &mut BranchSerializer {
    type Ok = ();
    type Error = AtomSerializeError;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
