use std::{io::Write, mem::size_of_val};

use anyhow::Result;
use serde::{
    ser::{SerializeSeq, SerializeTuple},
    Serialize, Serializer,
};

use crate::pcp::atom::serializer::data_serializer::DataSerializer;

use super::{
    helpers::UnreachableSerializer, serialize_parent_struct::SerializeParentStruct,
    AtomSerializeError,
};

#[derive(derive_new::new)]
pub struct BranchSerializer<'a, W: Write> {
    writer: W,
    identifier: &'a [u8; 4],
    children_count: usize,
}

impl<'a, W: Write> Serializer for BranchSerializer<'a, W> {
    type Ok = ();
    type Error = AtomSerializeError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = UnreachableSerializer;
    type SerializeTupleVariant = UnreachableSerializer;
    type SerializeMap = UnreachableSerializer;
    type SerializeStruct = SerializeParentStruct<W>;
    type SerializeStructVariant = UnreachableSerializer;

    fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_u8(mut self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(self.identifier)
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        self.writer
            .write_all(&1u32.to_le_bytes())
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        self.writer
            .write_all(&v.to_le_bytes())
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        Ok(())
    }

    fn serialize_u16(mut self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(self.identifier)
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        self.writer
            .write_all(&2u32.to_le_bytes())
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        self.writer
            .write_all(&v.to_le_bytes())
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        Ok(())
    }

    fn serialize_u32(mut self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(self.identifier)
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        self.writer
            .write_all(&4u32.to_le_bytes())
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        self.writer
            .write_all(&v.to_le_bytes())
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        Ok(())
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_str(self, _v: &str) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_seq(mut self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        // Vec<u8> など
        self.writer
            .write_all(self.identifier)
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        self.writer
            .write_all(&(len.unwrap() as u32).to_le_bytes())
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        Ok(self)
    }

    fn serialize_tuple(mut self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        // [u8; 16] など
        self.writer
            .write_all(self.identifier)
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        self.writer
            .write_all(&(len as u32).to_le_bytes())
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        Ok(self)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        unreachable!()
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        unreachable!()
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        unreachable!()
    }

    fn serialize_struct(
        mut self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.writer
            .write_all(self.identifier)
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        let length = 0x80000000u32 | self.children_count as u32;
        self.writer
            .write_all(&length.to_le_bytes())
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        Ok(SerializeParentStruct::new(self.writer))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        unreachable!()
    }
}

impl<'a, W: Write> SerializeSeq for BranchSerializer<'a, W> {
    type Ok = ();

    type Error = AtomSerializeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut DataSerializer::new(&mut self.writer))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a, W: Write> SerializeTuple for BranchSerializer<'a, W> {
    type Ok = ();

    type Error = AtomSerializeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        debug_assert_eq!(size_of_val(value), 1);
        value.serialize(&mut DataSerializer::new(&mut self.writer))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
