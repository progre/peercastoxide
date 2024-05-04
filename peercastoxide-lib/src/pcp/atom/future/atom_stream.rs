use anyhow::bail;
use anyhow::{anyhow, Result};
use async_recursion::async_recursion;
use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;
use tracing::trace;

use crate::pcp::atom::unknown::UnknownAtom;

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

    pub async fn read_atom(&mut self) -> Result<Option<UnknownAtom>> {
        self.read_atom_recursive().await
    }

    #[async_recursion]
    async fn read_atom_recursive(&mut self) -> Result<Option<UnknownAtom>> {
        let mut identifier = [0u8; 4];
        let n = self.stream.read_exact(&mut identifier).await?;
        if n == 0 {
            return Ok(None);
        }
        if n != 4 {
            bail!("invalid atom")
        }
        let length_src = self.stream.read_u32_le().await?;
        let is_parent = length_src & 0x80000000 != 0;
        let length = length_src & 0x7fffffff;
        if length > 1024 * 1024 {
            trace!(
                "broken id: {}, length: {}",
                identifier
                    .iter()
                    .map(|&x| (x as char).to_string())
                    .collect::<Vec<_>>()
                    .join(""),
                length
            );
            bail!("length too long")
        }
        if is_parent {
            let mut contents = Vec::with_capacity(length as usize);
            for _ in 0..length {
                contents.push(
                    self.read_atom_recursive()
                        .await?
                        .ok_or_else(|| anyhow!("invalid atom"))?,
                );
            }
            return Ok(Some(UnknownAtom::parent(identifier, contents)));
        }
        let mut buf = vec![0; length as usize];
        self.stream.read_exact(buf.as_mut()).await?;
        Ok(Some(UnknownAtom::child(identifier, buf)))
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

    pub async fn write_atom(&mut self, atom: &UnknownAtom) -> Result<()> {
        self.write_atom_recursive(atom).await
    }

    #[async_recursion]
    async fn write_atom_recursive(&mut self, atom: &UnknownAtom) -> Result<()> {
        self.stream.write_all(atom.identifier()).await?;
        match atom {
            UnknownAtom::Parent(parent) => {
                let length = 0x80000000u32 | parent.children().len() as u32;
                self.stream.write_u32_le(length).await?;
                for child in parent.children() {
                    self.write_atom_recursive(child).await?;
                }
                Ok(())
            }
            UnknownAtom::Child(child) => {
                self.stream.write_u32_le(child.data().len() as u32).await?;
                self.stream.write_all(child.data()).await?;
                Ok(())
            }
        }
    }
}
