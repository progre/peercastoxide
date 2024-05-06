mod atom_stream;
mod de;
mod ser;
mod unknown;
pub mod values;
pub mod well_known_atoms;
pub mod well_known_protocols;

pub use atom_stream::{AtomStreamReader, AtomStreamWriter};
pub use de::from_reader;
pub use ser::to_writer;
pub use unknown::{AtomChild, AtomParent, UnknownAtom};

fn to_string_without_zero_padding(string: &[u8]) -> String {
    String::from_utf8_lossy(string)
        .trim_end_matches('\0')
        .to_string()
}

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
