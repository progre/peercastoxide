use std::io::Read;

use serde::de::{DeserializeSeed, SeqAccess};

use crate::pcp::atom::de::{atom_buf_reader::AtomBufReader, AtomDeserializeError};

use super::deserializer::GroupedAtomsDeserializer;

pub struct GroupedAtomsSeqAccess<'a, R: Read> {
    de: GroupedAtomsDeserializer<'a, R>,
    first_atom: &'a [u8; 4],
}

impl<'a, R: Read> GroupedAtomsSeqAccess<'a, R> {
    pub fn new(
        reader: &'a mut AtomBufReader<R>,
        grouped_atoms: &'a [[u8; 4]],
        remaining: &'a mut u32,
    ) -> Self {
        Self {
            de: GroupedAtomsDeserializer::new(reader, grouped_atoms, remaining),
            first_atom: &grouped_atoms[0],
        }
    }
}

impl<'a, 'de, R: Read> SeqAccess<'de> for GroupedAtomsSeqAccess<'a, R> {
    type Error = AtomDeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, AtomDeserializeError>
    where
        T: DeserializeSeed<'de>,
    {
        if !self.de.has_remaining() {
            return Ok(None);
        }
        let identifier = self.de.reader_mut().peek_identifier()?;
        if self.first_atom != &identifier {
            return Ok(None);
        }
        seed.deserialize(&mut self.de).map(Some)
    }
}
