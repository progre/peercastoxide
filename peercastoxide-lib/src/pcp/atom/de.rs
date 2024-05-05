mod atom_buf_reader;
mod branch_deserializer;
mod byte_buf_deserializer;
mod children_map_access;
mod grouped_atoms;
mod helpers;
mod root_deserializer;
mod vec_data_seq_access;

use std::{
    fmt::Display,
    io::{self, Read},
};

use anyhow::{anyhow, Result};
use serde::Deserialize;

use self::root_deserializer::RootDeserializer;

/*
 * from_reader() デシリアライズ処理開始
 * Deserialize::deserialize() 出力先の型情報を確認
 * AtomDeserializer::deserialize_?() (child なら newtype_struct、parent なら struct) parent なら children をパースするために visit_map() を呼び出す
 * AtomChildrenMapAccess::next_key_seed() まだ要素があるか確認
 * AtomDeserializer::deserialize_identifier() 識別子を取得
 * DeserializeSeed::deserialize() map 構造を前提として型情報を再確認
 * AtomDeserializer::deserialize_?() ...
 */

#[derive(Debug, thiserror::Error)]
pub enum AtomDeserializeError {
    #[error("Atom deserialize unsupported structure")]
    UnsupportedStructure(#[source] anyhow::Error),
    #[error("Atom deserialize io error")]
    Io(#[from] io::Error),
    #[error("Atom deserialize mismatch")]
    Mismatch(#[source] anyhow::Error),
    #[error("Atom deserialize error: {0}")]
    Serde(String),
}

impl AtomDeserializeError {
    pub fn unsupported_structure(structure: &'static str) -> Self {
        let err = anyhow!("{} is not supported", structure);
        AtomDeserializeError::UnsupportedStructure(err)
    }
}

impl serde::de::Error for AtomDeserializeError {
    fn custom<T: Display>(msg: T) -> Self {
        AtomDeserializeError::Serde(msg.to_string())
    }
}

pub fn from_reader<'de, T: Deserialize<'de>>(
    reader: &mut impl Read,
) -> Result<T, AtomDeserializeError> {
    T::deserialize(RootDeserializer::new(reader))
}
