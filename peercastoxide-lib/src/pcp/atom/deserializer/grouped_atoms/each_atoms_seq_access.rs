use std::io::Read;

use serde::de::{DeserializeSeed, SeqAccess};

use crate::pcp::atom::deserializer::{
    atom_buf_reader::AtomBufReader, branch_deserializer::BranchDeserializer,
    raw_identifier_to_string, AtomDeserializeError,
};

pub struct GroupedAtomsEachAtomSeqAccess<'a, R: Read> {
    de: BranchDeserializer<'a, R>,
    grouped_atoms: &'a [[u8; 4]],
    idx: usize,
    remaining: &'a mut u32,
}

impl<'a, R: Read> GroupedAtomsEachAtomSeqAccess<'a, R> {
    pub fn new(
        reader: &'a mut AtomBufReader<R>,
        grouped_atoms: &'a [[u8; 4]],
        remaining: &'a mut u32,
    ) -> Self {
        Self {
            de: BranchDeserializer::new(reader, vec![], grouped_atoms.len() as u32),
            grouped_atoms,
            idx: 0,
            remaining,
        }
    }
}

impl<'a, 'de, R: Read> SeqAccess<'de> for GroupedAtomsEachAtomSeqAccess<'a, R> {
    type Error = AtomDeserializeError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, AtomDeserializeError>
    where
        T: DeserializeSeed<'de>,
    {
        debug_assert!(self.idx < self.grouped_atoms.len());
        if *self.remaining == 0 {
            return Err(AtomDeserializeError::Mismatch(format!(
                "identifier is expected {} but got nothing",
                raw_identifier_to_string(self.grouped_atoms[self.idx])
            )));
        }
        let identifier = self.de.reader_mut().read_identifier()?;
        if self.grouped_atoms[self.idx] != identifier {
            return Err(AtomDeserializeError::Mismatch(format!(
                "identifier is expected {} but got {}",
                raw_identifier_to_string(self.grouped_atoms[self.idx]),
                raw_identifier_to_string(identifier)
            )));
        }
        self.idx += 1;
        *self.remaining -= 1;
        seed.deserialize(&mut self.de).map(Some)
    }
}
