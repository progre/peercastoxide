use std::borrow::Cow;

use anyhow::anyhow;
use anyhow::Result;
use async_recursion::async_recursion;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;

use super::atom::Atom;
use super::atom::AtomChild;
use super::atom::AtomParent;

pub struct AtomStreamReader<T>
where
    T: AsyncRead + Unpin + Send + Sync,
{
    stream: T,
}

impl<T> AtomStreamReader<T>
where
    T: AsyncRead + Unpin + Send + Sync,
{
    pub fn new(stream: T) -> Self {
        Self { stream }
    }

    #[async_recursion]
    pub async fn read(&mut self) -> Result<Option<Atom>> {
        let mut identifier = [0u8; 4];
        let n = self.stream.read_exact(&mut identifier).await?;
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
            log::trace!(
                "broken id: {}, length: {}",
                identifier
                    .iter()
                    .map(|&x| (x as char).to_string())
                    .collect::<Vec<_>>()
                    .join(""),
                length
            );
            return Err(anyhow!("length too long"));
        }
        if is_parent {
            let mut contents = Vec::with_capacity(length as usize);
            for _ in 0..length {
                contents.push(self.read().await?.ok_or_else(|| anyhow!("invalid atom"))?);
            }
            return Ok(Some(Atom::Parent(AtomParent::new(
                Cow::Owned(identifier),
                contents,
            ))));
        }
        let mut buf: Vec<u8> = Vec::new();
        buf.resize(length as usize, 0);
        self.stream.read_exact(buf.as_mut()).await?;
        Ok(Some(Atom::Child(AtomChild::new(
            Cow::Owned(identifier),
            buf,
        ))))
    }
}

pub struct AtomStreamWriter<T>
where
    T: AsyncWrite + Unpin + Send + Sync,
{
    stream: T,
}

impl<T> AtomStreamWriter<T>
where
    T: AsyncWrite + Unpin + Send + Sync,
{
    pub fn new(stream: T) -> Self {
        Self { stream }
    }

    #[async_recursion]
    pub async fn write(&mut self, atom: &Atom) -> Result<()> {
        self.stream.write_all(atom.identifier()).await?;
        match atom {
            Atom::Parent(parent) => {
                let length = 0x80000000u32 | parent.children().len() as u32;
                self.stream.write_u32_le(length).await?;
                for child in parent.children() {
                    self.write(child).await?;
                }
                Ok(())
            }
            Atom::Child(child) => {
                self.stream.write_u32_le(child.data().len() as u32).await?;
                self.stream.write_all(child.data()).await?;
                Ok(())
            }
        }
    }
}
