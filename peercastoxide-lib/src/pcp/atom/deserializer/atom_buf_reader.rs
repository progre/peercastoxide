use std::io::Read;

use anyhow::Result;

use super::AtomDeserializeError;

#[derive(getset::MutGetters)]
pub struct AtomBufReader<R: Read> {
    #[get_mut = "pub"]
    reader: R,
    peeked_identifier: Option<[u8; 4]>,
}

impl<R: Read> AtomBufReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            peeked_identifier: None,
        }
    }

    pub fn into_reader(self) -> R {
        self.reader
    }

    pub fn peek_identifier(&mut self) -> Result<[u8; 4], AtomDeserializeError> {
        if let Some(identifier) = self.peeked_identifier.as_ref() {
            return Ok(*identifier);
        }
        let identifier = self.read_identifier()?;
        self.peeked_identifier = Some(identifier);
        Ok(identifier)
    }

    pub fn read_identifier(&mut self) -> Result<[u8; 4], AtomDeserializeError> {
        if let Some(identifier) = self.peeked_identifier.take() {
            return Ok(identifier);
        }
        let mut identifier = [0u8; 4];
        self.read_exact(&mut identifier)?;
        Ok(identifier)
    }

    fn read_length(&mut self) -> Result<(bool, u32), AtomDeserializeError> {
        let length_src = self.read_u32()?;
        let is_parent = length_src & 0x80000000 != 0;
        let length = length_src & 0x7fffffff;
        if length > 1024 * 1024 {
            let msg = format!("length too large: {}", length);
            return Err(AtomDeserializeError::Mismatch(msg));
        }
        Ok((is_parent, length))
    }

    pub fn read_children_count(&mut self) -> Result<u32, AtomDeserializeError> {
        let (is_parent, count) = self.read_length()?;
        if !is_parent {
            const MSG: &str = "atom is expected parent but got child";
            return Err(AtomDeserializeError::Mismatch(MSG.into()));
        }
        Ok(count)
    }

    pub fn read_data_size(&mut self) -> Result<u32, AtomDeserializeError> {
        let (is_parent, size) = self.read_length()?;
        if is_parent {
            const MSG: &str = "atom is expected child but got parent";
            return Err(AtomDeserializeError::Mismatch(MSG.into()));
        }
        Ok(size)
    }

    pub fn read_u8(&mut self) -> Result<u8, AtomDeserializeError> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    pub fn read_u16(&mut self) -> Result<u16, AtomDeserializeError> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    pub fn read_u32(&mut self) -> Result<u32, AtomDeserializeError> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    pub fn read_u128(&mut self) -> Result<u128, AtomDeserializeError> {
        let mut buf = [0u8; 16];
        self.read_exact(&mut buf)?;
        Ok(u128::from_le_bytes(buf))
    }

    pub fn read_data_size_and_byte_buf(&mut self) -> Result<Vec<u8>, AtomDeserializeError> {
        let size = self.read_data_size()?;
        let mut buf = vec![0u8; size as usize];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<(), AtomDeserializeError> {
        self.reader
            .read_exact(buf)
            .map_err(AtomDeserializeError::Io)
    }
}
