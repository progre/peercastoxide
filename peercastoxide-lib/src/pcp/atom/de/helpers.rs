#[macro_export]
macro_rules! common_unsupported_deserializes {
    () => {
        fn deserialize_bool<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("bool"))
        }
        fn deserialize_i8<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("i8"))
        }
        fn deserialize_i16<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("i16"))
        }
        fn deserialize_i32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("i32"))
        }
        fn deserialize_i64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("i64"))
        }
        fn deserialize_u64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("u64"))
        }
        fn deserialize_f32<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("f32"))
        }
        fn deserialize_f64<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("f64"))
        }
        fn deserialize_char<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("char"))
        }
        fn deserialize_str<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("str"))
        }
        fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("bytes"))
        }
        fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("byte buf"))
        }
        fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("unit"))
        }
        fn deserialize_unit_struct<V>(
            self,
            _name: &'static str,
            _visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("unit struct"))
        }
        fn deserialize_tuple_struct<V>(
            self,
            _name: &'static str,
            _len: usize,
            _visitor: V,
        ) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("tuple struct"))
        }
        fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("map"))
        }
        fn deserialize_ignored_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            Err(AtomDeserializeError::unsupported_structure("ignored_any"))
        }
    };
}
