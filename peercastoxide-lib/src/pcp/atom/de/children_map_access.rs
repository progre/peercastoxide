use std::io::Read;

use anyhow::Result;
use serde::de::{DeserializeSeed, MapAccess};

use crate::pcp::atom::{is_grouped_atoms, to_grouped_atoms};

use super::{
    atom_buf_reader::AtomBufReader, branch_deserializer::BranchDeserializer, AtomDeserializeError,
};

fn find_grouped_atoms(fields: &[&str]) -> Vec<Vec<[u8; 4]>> {
    fields
        .iter()
        .filter(|x| is_grouped_atoms(x))
        .map(|x| to_grouped_atoms(x))
        .collect()
}

pub struct ChildrenMapAccess<'a, R: Read> {
    de: BranchDeserializer<'a, R>,
}

impl<'a, R: Read> ChildrenMapAccess<'a, R> {
    pub fn new(reader: &'a mut AtomBufReader<R>, fields: &[&str], remaining: u32) -> Self {
        let grouped_atoms_list = find_grouped_atoms(fields);
        let de = BranchDeserializer::new(reader, grouped_atoms_list, remaining);
        Self { de }
    }
}

impl<'a, 'de, R: Read> MapAccess<'de> for ChildrenMapAccess<'a, R> {
    type Error = AtomDeserializeError;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, AtomDeserializeError>
    where
        K: DeserializeSeed<'de>,
    {
        if self.de.remaining() == 0 {
            return Ok(None);
        }
        seed.deserialize(&mut self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, AtomDeserializeError>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut self.de)
    }
}
