use std::io::Write;

use serde::{Serialize, Serializer};

use super::{helpers::UnreachableSerializer, AtomSerializeError};

#[derive(derive_new::new)]
pub struct DataSerializer<W: Write> {
    writer: W,
}

impl<W: Write> Serializer for &mut DataSerializer<W> {
    type Ok = ();
    type Error = AtomSerializeError;
    type SerializeSeq = UnreachableSerializer;
    type SerializeTuple = UnreachableSerializer;
    type SerializeTupleStruct = UnreachableSerializer;
    type SerializeTupleVariant = UnreachableSerializer;
    type SerializeMap = UnreachableSerializer;
    type SerializeStruct = UnreachableSerializer;
    type SerializeStructVariant = UnreachableSerializer;

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.writer
            .write_all(&v.to_le_bytes())
            .map_err(|e| AtomSerializeError::Io(format!("failed to write: {}", e)))?;
        Ok(())
    }

    // ----

    // unreachable

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

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        unreachable!()
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
        unreachable!()
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
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
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
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

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        unreachable!()
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        unreachable!()
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
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        unreachable!()
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
