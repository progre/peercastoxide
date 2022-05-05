use crate::pcp::atom::Atom;
use crate::pcp::atom::AtomContent;
use anyhow::anyhow;
use anyhow::Result;
use log::*;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;

pub struct AtomStreamReader<T>
where
    T: AsyncRead + Unpin,
{
    stream: T,
}

impl<T> AtomStreamReader<T>
where
    T: AsyncRead + Unpin,
{
    pub fn new(stream: T) -> Self {
        Self { stream }
    }

    pub async fn read(&mut self) -> Result<Option<Atom>> {
        let mut identifier = [0u8; 4];
        let n = self.stream.read(&mut identifier).await?;
        if n == 0 {
            return Ok(None);
        }
        if n != 4 {
            return Err(anyhow!("invalid atom"));
        }
        let length_src = self.stream.read_u32_le().await?;
        let is_parent = length_src & 0x80000000 != 0;
        let length = length_src & 0x7fffffff;
        if length > 1024 * 1024 {
            trace!("broken");
            return Err(anyhow!("length too long"));
        }
        if is_parent {
            return Ok(Some(Atom::parent(identifier, length as i32)));
        }
        let mut buf: Vec<u8> = Vec::new();
        buf.resize(length as usize, 0);
        self.stream.read(buf.as_mut()).await?;
        Ok(Some(Atom::child(identifier, buf)))
    }
}

pub struct AtomStreamWriter<T>
where
    T: AsyncWrite + Unpin,
{
    stream: T,
}

impl<T> AtomStreamWriter<T>
where
    T: AsyncWrite + Unpin,
{
    pub fn new(stream: T) -> Self {
        Self { stream }
    }

    pub async fn write(&mut self, atom: &Atom) -> Result<()> {
        self.stream.write(atom.identifier()).await?;
        match atom.content() {
            AtomContent::Parent(parent) => {
                let length = 0x80000000u32 | parent.count() as u32;
                self.stream.write_u32_le(length).await?;
                Ok(())
            }
            AtomContent::Child(child) => {
                self.stream.write_u32_le(child.data().len() as u32).await?;
                self.stream.write(child.data()).await?;
                Ok(())
            }
        }
    }
}
