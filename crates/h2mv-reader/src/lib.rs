//! Safe, positioned read-only access to immutable H2 MVStore files.

mod error;
mod io;
mod options;

pub use error::ReaderError;
pub use io::{FileIdentity, IoStatsSnapshot, PlatformFileId, ReadAt, ReadOnlySingleFileStore};
pub use options::{LockPolicy, OpenOptions};
