use std::io::Write;

use serde::{ser::SerializeSeq, Serialize};

use crate::pcp::atom::ser::AtomSerializeError;

use super::serializer::GroupedAtomsSerializer;

pub struct SerializeGroupedAtomsSeq<'a, W: Write> {
    writer: W,
    grouped_atoms: &'a [[u8; 4]],
}

impl<'a, W: Write> SerializeGroupedAtomsSeq<'a, W> {
    pub fn new(writer: W, grouped_atoms: &'a [[u8; 4]]) -> Self {
        Self {
            writer,
            grouped_atoms,
        }
    }
}

impl<'a, W: Write> SerializeSeq for SerializeGroupedAtomsSeq<'a, W> {
    type Ok = ();
    type Error = AtomSerializeError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let ser = GroupedAtomsSerializer::new(&mut self.writer, self.grouped_atoms);
        value.serialize(ser)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
