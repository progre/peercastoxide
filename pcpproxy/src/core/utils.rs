use std::io;

use thiserror::Error;

use crate::features::output::ndjson::NDJson;

#[derive(Error, Debug)]
pub enum PipeError {
    #[error(transparent)]
    DisconnectedByIncoming(anyhow::Error),
    #[error(transparent)]
    DisconnectedByOutgoing(anyhow::Error),
}

pub fn is_broken_pipe_error(err: &anyhow::Error) -> bool {
    if let Some(err) = err.downcast_ref::<io::Error>() {
        if err.kind() == io::ErrorKind::ConnectionReset {
            return true;
        }
    }
    false
}

pub fn disconnect_conn_of_upload(
    result: Result<(), PipeError>,
    output: NDJson,
) -> anyhow::Result<()> {
    match &result {
        Err(PipeError::DisconnectedByIncoming(err)) => {
            if is_broken_pipe_error(err) {
                output.disconnected_by_client();
                return Ok(());
            }
            eprintln!("Unknown error: {}", err);
            result.map_err(anyhow::Error::new)
        }
        Err(PipeError::DisconnectedByOutgoing(err)) => {
            if is_broken_pipe_error(err) {
                output.disconnected_by_server();
                return Ok(());
            }
            eprintln!("Unknown error: {}", err);
            result.map_err(anyhow::Error::new)
        }
        Ok(_) => Ok(()),
    }
}

pub fn disconnect_conn_of_download(
    result: Result<(), PipeError>,
    output: NDJson,
) -> anyhow::Result<()> {
    match &result {
        Err(PipeError::DisconnectedByIncoming(err)) => {
            if is_broken_pipe_error(err) {
                output.disconnected_by_server();
                return Ok(());
            }
            eprintln!("Unknown error: {}", err);
            result.map_err(anyhow::Error::new)
        }
        Err(PipeError::DisconnectedByOutgoing(err)) => {
            if is_broken_pipe_error(err) {
                output.disconnected_by_client();
                return Ok(());
            }
            eprintln!("Unknown error: {}", err);
            result.map_err(anyhow::Error::new)
        }
        Ok(_) => Ok(()),
    }
}
