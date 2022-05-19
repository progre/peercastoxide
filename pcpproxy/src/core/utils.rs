use std::io::{self, ErrorKind};

use thiserror::Error;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWriteExt},
    net::tcp::OwnedWriteHalf,
};

use crate::features::output::ndjson::NDJson;

use super::pcp_proxy::pipe::big_vec;

#[derive(Error, Debug)]
pub enum PipeError {
    #[error(transparent)]
    ByIncoming(anyhow::Error),
    #[error(transparent)]
    ByOutgoing(anyhow::Error),
}

async fn pipe_one_read(
    incoming: &mut (impl AsyncRead + Unpin),
    outgoing: &mut OwnedWriteHalf,
    buf: &mut [u8],
) -> Result<bool, PipeError> {
    let n = incoming
        .read(buf)
        .await
        .map_err(|err| PipeError::ByIncoming(anyhow::Error::new(err)))?;
    if n == 0 {
        return Ok(false);
    }
    outgoing
        .write_all(&buf[0..n])
        .await
        .map_err(|err| PipeError::ByOutgoing(anyhow::Error::new(err)))?;
    Ok(true)
}

pub async fn pipe_raw(
    mut incoming: impl AsyncRead + Unpin,
    mut outgoing: OwnedWriteHalf,
    output: &NDJson,
) -> Result<(), PipeError> {
    let mut buf = big_vec(1024 * 1024);
    if !pipe_one_read(&mut incoming, &mut outgoing, &mut buf).await? {
        return Ok(());
    }
    output.output_raw("(raw data stream)");
    loop {
        if !pipe_one_read(&mut incoming, &mut outgoing, &mut buf).await? {
            return Ok(());
        }
    }
}

pub fn io_error_kind(err: &anyhow::Error) -> Option<ErrorKind> {
    err.downcast_ref::<io::Error>().map(|err| err.kind())
}

pub fn disconnect_conn_of_upload(
    result: Result<(), PipeError>,
    output: NDJson,
) -> anyhow::Result<()> {
    match &result {
        Err(PipeError::ByIncoming(err)) => {
            if let Some(error_kind) = io_error_kind(err) {
                output.disconnected_by_client(Some(error_kind));
                return Ok(());
            }
            eprintln!("Unknown error: {}", err);
            result.map_err(anyhow::Error::new)
        }
        Err(PipeError::ByOutgoing(err)) => {
            if let Some(error_kind) = io_error_kind(err) {
                output.disconnected_by_server(Some(error_kind));
                return Ok(());
            }
            eprintln!("Unknown error: {}", err);
            result.map_err(anyhow::Error::new)
        }
        Ok(_) => {
            output.disconnected_by_client(None);
            Ok(())
        }
    }
}

pub fn disconnect_conn_of_download(
    result: Result<(), PipeError>,
    output: NDJson,
) -> anyhow::Result<()> {
    match &result {
        Err(PipeError::ByIncoming(err)) => {
            if let Some(error_kind) = io_error_kind(err) {
                output.disconnected_by_server(Some(error_kind));
                return Ok(());
            }
            eprintln!("Unknown error: {}", err);
            result.map_err(anyhow::Error::new)
        }
        Err(PipeError::ByOutgoing(err)) => {
            if let Some(error_kind) = io_error_kind(err) {
                output.disconnected_by_client(Some(error_kind));
                return Ok(());
            }
            eprintln!("Unknown error: {}", err);
            result.map_err(anyhow::Error::new)
        }
        Ok(_) => {
            output.disconnected_by_server(None);
            Ok(())
        }
    }
}
