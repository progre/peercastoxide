use std::io::Read;

use serde::{de::Visitor, forward_to_deserialize_any, Deserializer};

use crate::pcp::atom::deserializer::{atom_buf_reader::AtomBufReader, AtomDeserializeError};

use super::each_atoms_seq_access::GroupedAtomsEachAtomSeqAccess;

#[derive(getset::Getters, getset::MutGetters)]
pub struct GroupedAtomsDeserializer<'a, R: Read> {
    #[get_mut = "pub"]
    reader: &'a mut AtomBufReader<R>,
    grouped_atoms: &'a [[u8; 4]],
    remaining: &'a mut u32,
}

impl<'a, R: Read> GroupedAtomsDeserializer<'a, R> {
    pub fn new(
        reader: &'a mut AtomBufReader<R>,
        grouped_atoms: &'a [[u8; 4]],
        remaining: &'a mut u32,
    ) -> Self {
        Self {
            reader,
            grouped_atoms,
            remaining,
        }
    }

    pub fn has_remaining(&self) -> bool {
        *self.remaining > 0
    }
}

impl<'a, 'de, R: Read> Deserializer<'de> for &mut GroupedAtomsDeserializer<'a, R> {
    type Error = AtomDeserializeError;

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let seq =
            GroupedAtomsEachAtomSeqAccess::new(self.reader, self.grouped_atoms, self.remaining);
        visitor.visit_seq(seq)
    }

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        unreachable!()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        map struct enum identifier ignored_any
    }
}
