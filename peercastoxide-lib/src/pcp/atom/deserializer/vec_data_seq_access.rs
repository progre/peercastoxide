use serde::de::SeqAccess;

use super::{byte_buf_deserializer::ByteBufDeserializer, AtomDeserializeError};

pub struct VecDataSeqAccess {
    de: ByteBufDeserializer,
}

impl VecDataSeqAccess {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            de: ByteBufDeserializer::new(data),
        }
    }
}

impl<'de> SeqAccess<'de> for VecDataSeqAccess {
    type Error = AtomDeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: serde::de::DeserializeSeed<'de>,
    {
        if !self.de.has_remaining() {
            return Ok(None);
        }
        seed.deserialize(&mut self.de).map(Some)
    }
}
