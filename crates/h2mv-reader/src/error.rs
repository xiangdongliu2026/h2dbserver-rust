use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReaderError {
    #[error("I/O failure at byte offset {offset}: {source}")]
    Io {
        offset: u64,
        #[source]
        source: io::Error,
    },
    #[error("unexpected EOF at byte offset {offset}: wanted {wanted} bytes")]
    UnexpectedEof { offset: u64, wanted: usize },
    #[error("path is not a regular file: {path}")]
    NotRegularFile { path: PathBuf },
    #[error("unable to obtain a shared file lock")]
    FileLockUnavailable,
    #[error("database file changed while it was open")]
    FileChanged,
    #[error("corrupt MVStore data: {message}")]
    CorruptData { message: String },
}
