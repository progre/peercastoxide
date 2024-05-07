mod atom_stream;
mod de;
mod ser;
mod unknown;
pub mod values;
pub mod well_known_atoms;
pub mod well_known_identifiers;
pub mod well_known_protocols;

pub use atom_stream::{AtomStreamReader, AtomStreamWriter};
pub use de::from_reader;
pub use ser::to_writer;
pub use unknown::{from_unknown, to_unknown, AtomChild, AtomParent, UnknownAtom};

fn is_grouped_atoms(identifier: &str) -> bool {
    identifier.len() / 4 >= 2
}

fn to_grouped_atoms(identifier: &str) -> Vec<[u8; 4]> {
    identifier
        .bytes()
        .collect::<Vec<_>>()
        .chunks(4)
        .map(|x| {
            let mut buf = [0u8; 4];
            buf.copy_from_slice(x);
            buf
        })
        .collect()
}
