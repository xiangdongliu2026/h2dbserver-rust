use crate::{LockPolicy, ReaderError};
use fs2::FileExt as LockExt;
use std::{
    fs::{File, OpenOptions as FsOpenOptions},
    os::unix::{
        fs::{FileExt, MetadataExt},
        io::AsRawFd,
    },
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};

pub trait ReadAt: Send + Sync + 'static {
    fn len(&self) -> Result<u64, ReaderError>;
    fn read_exact_at(&self, offset: u64, destination: &mut [u8]) -> Result<(), ReaderError>;
    fn is_empty(&self) -> Result<bool, ReaderError> {
        Ok(self.len()? == 0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlatformFileId {
    pub device: u64,
    pub inode: u64,
}
#[derive(Clone, Debug)]
pub struct FileIdentity {
    pub length: u64,
    pub modified_at: Option<SystemTime>,
    pub platform_id: PlatformFileId,
}
impl FileIdentity {
    fn from_file(file: &File) -> Result<Self, ReaderError> {
        let m = file
            .metadata()
            .map_err(|source| ReaderError::Io { offset: 0, source })?;
        Ok(Self {
            length: m.len(),
            modified_at: m.modified().ok(),
            platform_id: PlatformFileId {
                device: m.dev(),
                inode: m.ino(),
            },
        })
    }
}

#[derive(Default)]
struct IoStats {
    operations: AtomicU64,
    bytes: AtomicU64,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct IoStatsSnapshot {
    pub operations: u64,
    pub bytes: u64,
}
struct FileLockGuard {
    file: File,
}
impl Drop for FileLockGuard {
    fn drop(&mut self) {
        let _ = LockExt::unlock(&self.file);
    }
}

pub struct ReadOnlySingleFileStore {
    file: File,
    path: PathBuf,
    identity: FileIdentity,
    opened_length: u64,
    lock: Option<FileLockGuard>,
    stats: IoStats,
}
impl ReadOnlySingleFileStore {
    pub fn open(path: impl AsRef<Path>, policy: LockPolicy) -> Result<Self, ReaderError> {
        let path = path.as_ref().to_path_buf();
        let file = FsOpenOptions::new()
            .read(true)
            .open(&path)
            .map_err(|source| ReaderError::Io { offset: 0, source })?;
        let metadata = file
            .metadata()
            .map_err(|source| ReaderError::Io { offset: 0, source })?;
        if !metadata.is_file() {
            return Err(ReaderError::NotRegularFile { path });
        }
        let identity = FileIdentity::from_file(&file)?;
        let lock = if policy == LockPolicy::Disabled {
            None
        } else {
            let lock_file = file
                .try_clone()
                .map_err(|source| ReaderError::Io { offset: 0, source })?;
            match LockExt::try_lock_shared(&lock_file) {
                Ok(()) => Some(FileLockGuard { file: lock_file }),
                Err(_) if policy == LockPolicy::BestEffort => None,
                Err(_) => return Err(ReaderError::FileLockUnavailable),
            }
        };
        Ok(Self {
            file,
            path,
            opened_length: identity.length,
            identity,
            lock,
            stats: IoStats::default(),
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn identity(&self) -> &FileIdentity {
        &self.identity
    }
    pub fn has_lock(&self) -> bool {
        self.lock.is_some()
    }
    pub fn raw_fd(&self) -> i32 {
        self.file.as_raw_fd()
    }
    pub fn io_stats(&self) -> IoStatsSnapshot {
        IoStatsSnapshot {
            operations: self.stats.operations.load(Ordering::Relaxed),
            bytes: self.stats.bytes.load(Ordering::Relaxed),
        }
    }
    pub fn verify_unchanged(&self) -> Result<(), ReaderError> {
        let current = FileIdentity::from_file(&self.file)?;
        if current.length != self.identity.length
            || current.platform_id != self.identity.platform_id
            || current.modified_at != self.identity.modified_at
        {
            return Err(ReaderError::FileChanged);
        }
        Ok(())
    }
}
impl ReadAt for ReadOnlySingleFileStore {
    fn len(&self) -> Result<u64, ReaderError> {
        let length = self
            .file
            .metadata()
            .map_err(|source| ReaderError::Io { offset: 0, source })?
            .len();
        if length < self.opened_length {
            return Err(ReaderError::FileChanged);
        }
        Ok(length)
    }
    fn read_exact_at(
        &self,
        mut offset: u64,
        mut destination: &mut [u8],
    ) -> Result<(), ReaderError> {
        offset
            .checked_add(destination.len() as u64)
            .ok_or_else(|| ReaderError::CorruptData {
                message: "file range overflow".into(),
            })?;
        let wanted = destination.len();
        self.stats.operations.fetch_add(1, Ordering::Relaxed);
        while !destination.is_empty() {
            let count = self
                .file
                .read_at(destination, offset)
                .map_err(|source| ReaderError::Io { offset, source })?;
            if count == 0 {
                return Err(ReaderError::UnexpectedEof {
                    offset,
                    wanted: destination.len(),
                });
            }
            offset = offset
                .checked_add(count as u64)
                .ok_or_else(|| ReaderError::CorruptData {
                    message: "file offset overflow".into(),
                })?;
            self.stats.bytes.fetch_add(count as u64, Ordering::Relaxed);
            destination = &mut destination[count..];
        }
        debug_assert_eq!(
            wanted as u64,
            self.stats.bytes.load(Ordering::Relaxed).min(wanted as u64)
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn positioned_reads_and_eof_are_typed() {
        let path = std::env::temp_dir().join(format!("h2mv-read-{}", std::process::id()));
        std::fs::write(&path, b"abcdef").unwrap();
        let store = ReadOnlySingleFileStore::open(&path, LockPolicy::Disabled).unwrap();
        let mut bytes = [0; 3];
        store.read_exact_at(2, &mut bytes).unwrap();
        assert_eq!(&bytes, b"cde");
        assert!(matches!(
            store.read_exact_at(5, &mut bytes),
            Err(ReaderError::UnexpectedEof { .. })
        ));
        std::fs::remove_file(path).unwrap();
    }
}
