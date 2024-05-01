use std::io::Write;

use serde::{ser::SerializeStruct, Serialize};

use crate::pcp::atom::{
    is_grouped_atoms,
    serializer::{
        count_children::count_children, grouped_atoms::seq_serializer::GroupedAtomsSeqSerializer,
    },
    to_grouped_atoms,
};

use super::{branch_serializer::BranchSerializer, AtomSerializeError};

#[derive(derive_new::new)]
pub struct SerializeParentStruct<W: Write> {
    writer: W,
}

impl<W: Write> SerializeStruct for SerializeParentStruct<W> {
    type Ok = ();
    type Error = AtomSerializeError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        debug_assert!(key.is_ascii());
        if is_grouped_atoms(key) {
            let ga = to_grouped_atoms(key);
            return value.serialize(GroupedAtomsSeqSerializer::new(&mut self.writer, &ga));
        }

        let mut identifier = [0u8; 4];
        identifier.copy_from_slice(format!("{:\0<4}", key).as_bytes());

        value.serialize(BranchSerializer::new(
            &mut self.writer,
            &identifier,
            count_children(value)?,
        ))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}
