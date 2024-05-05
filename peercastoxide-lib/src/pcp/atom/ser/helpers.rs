use serde::{
    ser::{
        SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize,
};

use super::AtomSerializeError;

#[macro_export]
macro_rules! common_unsupported_serializes {
    () => {
        fn serialize_bool(self, _v: bool) -> Result<Self::Ok, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("bool"))
        }
        fn serialize_i8(self, _v: i8) -> Result<Self::Ok, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("i8"))
        }
        fn serialize_i16(self, _v: i16) -> Result<Self::Ok, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("i16"))
        }
        fn serialize_i32(self, _v: i32) -> Result<Self::Ok, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("i32"))
        }
        fn serialize_i64(self, _v: i64) -> Result<Self::Ok, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("i64"))
        }
        fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("u64"))
        }
        fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("f32"))
        }
        fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("f64"))
        }
        fn serialize_char(self, _v: char) -> Result<Self::Ok, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("char"))
        }
        fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("bytes"))
        }
        fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("unit"))
        }
        fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("unit struct"))
        }
        fn serialize_unit_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
        ) -> Result<Self::Ok, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("unit variant"))
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
            Err(AtomSerializeError::unsupported_structure("newtype variant"))
        }
        fn serialize_tuple_struct(
            self,
            _name: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleStruct, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("tuple struct"))
        }
        fn serialize_tuple_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeTupleVariant, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("tuple variant"))
        }
        fn serialize_struct_variant(
            self,
            _name: &'static str,
            _variant_index: u32,
            _variant: &'static str,
            _len: usize,
        ) -> Result<Self::SerializeStructVariant, Self::Error> {
            Err(AtomSerializeError::unsupported_structure("struct variant"))
        }
    };
}

pub struct UnreachableSerializer<Ok> {
    _phantom: std::marker::PhantomData<Ok>,
}

impl<Ok> SerializeSeq for UnreachableSerializer<Ok> {
    type Ok = Ok;
    type Error = AtomSerializeError;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<Ok> SerializeTuple for UnreachableSerializer<Ok> {
    type Ok = Ok;
    type Error = AtomSerializeError;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<Ok> SerializeTupleStruct for UnreachableSerializer<Ok> {
    type Ok = Ok;
    type Error = AtomSerializeError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<Ok> SerializeTupleVariant for UnreachableSerializer<Ok> {
    type Ok = Ok;
    type Error = AtomSerializeError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<Ok> SerializeMap for UnreachableSerializer<Ok> {
    type Ok = Ok;
    type Error = AtomSerializeError;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<Ok> SerializeStruct for UnreachableSerializer<Ok> {
    type Ok = Ok;
    type Error = AtomSerializeError;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}

impl<Ok> SerializeStructVariant for UnreachableSerializer<Ok> {
    type Ok = Ok;
    type Error = AtomSerializeError;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        unreachable!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        unreachable!()
    }
}
