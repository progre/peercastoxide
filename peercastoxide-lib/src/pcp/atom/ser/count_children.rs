mod branch_serializer;
mod root_serializer;

use serde::Serialize;

use self::root_serializer::RootSerializer;

use super::AtomSerializeError;

pub fn count_children(value: impl Serialize) -> Result<usize, AtomSerializeError> {
    let mut counter = RootSerializer::new();
    value.serialize(&mut counter)?;
    Ok(counter.result())
}
