use std::{ffi::CString, io::Write, mem::size_of_val};

use anyhow::Result;
use serde::{
    ser::{SerializeSeq, SerializeTuple},
    Serialize, Serializer,
};

use crate::{common_unsupported_serializes, pcp::atom::ser::data_serializer::DataSerializer};

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
    type SerializeTupleStruct = UnreachableSerializer<Self::Ok>;
    type SerializeTupleVariant = UnreachableSerializer<Self::Ok>;
    type SerializeMap = UnreachableSerializer<Self::Ok>;
    type SerializeStruct = SerializeParentStruct<W>;
    type SerializeStructVariant = UnreachableSerializer<Self::Ok>;

    common_unsupported_serializes! {}

    fn serialize_u8(mut self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(self.identifier)
            .map_err(AtomSerializeError::Io)?;
        self.writer
            .write_all(&1u32.to_le_bytes())
            .map_err(AtomSerializeError::Io)?;
        self.writer
            .write_all(&v.to_le_bytes())
            .map_err(AtomSerializeError::Io)?;
        Ok(())
    }

    fn serialize_u16(mut self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(self.identifier)
            .map_err(AtomSerializeError::Io)?;
        self.writer
            .write_all(&2u32.to_le_bytes())
            .map_err(AtomSerializeError::Io)?;
        self.writer
            .write_all(&v.to_le_bytes())
            .map_err(AtomSerializeError::Io)?;
        Ok(())
    }

    fn serialize_u32(mut self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(self.identifier)
            .map_err(AtomSerializeError::Io)?;
        self.writer
            .write_all(&4u32.to_le_bytes())
            .map_err(AtomSerializeError::Io)?;
        self.writer
            .write_all(&v.to_le_bytes())
            .map_err(AtomSerializeError::Io)?;
        Ok(())
    }

    fn serialize_str(mut self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(self.identifier)
            .map_err(AtomSerializeError::Io)?;
        let c_string =
            CString::new(v).map_err(|e| AtomSerializeError::UnsupportedStructure(e.into()))?;
        let buf: &[u8] = c_string.as_bytes_with_nul();
        self.writer
            .write_all(&(buf.len() as u32).to_le_bytes())
            .map_err(AtomSerializeError::Io)?;
        self.writer.write_all(buf).map_err(AtomSerializeError::Io)?;
        Ok(())
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

    fn serialize_seq(mut self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        // Vec<u8> など
        self.writer
            .write_all(self.identifier)
            .map_err(AtomSerializeError::Io)?;
        self.writer
            .write_all(&(len.unwrap() as u32).to_le_bytes())
            .map_err(AtomSerializeError::Io)?;
        Ok(self)
    }

    fn serialize_tuple(mut self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        // [u8; 16] など
        self.writer
            .write_all(self.identifier)
            .map_err(AtomSerializeError::Io)?;
        self.writer
            .write_all(&(len as u32).to_le_bytes())
            .map_err(AtomSerializeError::Io)?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(AtomSerializeError::unsupported_structure("map"))
    }

    fn serialize_struct(
        mut self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.writer
            .write_all(self.identifier)
            .map_err(AtomSerializeError::Io)?;
        let length = 0x80000000u32 | self.children_count as u32;
        self.writer
            .write_all(&length.to_le_bytes())
            .map_err(AtomSerializeError::Io)?;
        Ok(SerializeParentStruct::new(self.writer))
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
