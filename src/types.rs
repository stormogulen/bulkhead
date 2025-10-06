//! Core types for the VFS, including Qids, Stats, and file handles.

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::time::SystemTime;

/// Marker types for files/dirs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct File;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dir;

/// Marker types for access modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadOnly;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteOnly;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadWrite;

/// Traits for read/write capabilities
pub trait CanRead: Send + Sync + 'static {}
impl CanRead for ReadOnly {}
impl CanRead for ReadWrite {}

pub trait CanWrite: Send + Sync + 'static {}
impl CanWrite for WriteOnly {}
impl CanWrite for ReadWrite {}

/// Enum for distinguishing file types at runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    File,
    Dir,
}

/// Unique file identifier (9P qid)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Qid {
    pub ty: u8,
    pub version: u32,
    pub path: u64,
}

impl Qid {
    pub const QTDIR: u8 = 0x80;
    pub const QTFILE: u8 = 0x00;

    pub fn new_file(path: u64, version: u32) -> Self {
        Self {
            ty: Self::QTFILE,
            version,
            path,
        }
    }
    pub fn new_dir(path: u64, version: u32) -> Self {
        Self {
            ty: Self::QTDIR,
            version,
            path,
        }
    }
    pub fn is_dir(&self) -> bool {
        self.ty & Self::QTDIR != 0
    }
    pub fn file_type(&self) -> FileType {
        if self.is_dir() {
            FileType::Dir
        } else {
            FileType::File
        }
    }
}

/// File metadata (9P stat)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stat {
    pub qid: Qid,
    pub name: String,
    pub size: u64,
    pub mode: u32,
    pub atime: SystemTime,
    pub mtime: SystemTime,
    pub uid: String,
    pub gid: String,
}

impl Stat {
    pub fn is_dir(&self) -> bool {
        self.qid.is_dir()
    }
    pub fn file_type(&self) -> FileType {
        self.qid.file_type()
    }
}

/// Open file/directory handle with type-level guarantees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHandle<T = File, M = ReadOnly> {
    pub fid: u64,
    pub qid: Qid,
    pub path: String,
    pub mode: u32,
    #[serde(skip)]
    _marker: PhantomData<(T, M)>,
}

impl<T, M> FileHandle<T, M> {
    pub fn new(fid: u64, qid: Qid, path: String, mode: u32) -> Self {
        Self {
            fid,
            qid,
            path,
            mode,
            _marker: PhantomData,
        }
    }
}

/// Context for authenticated operations (zero-trust)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VfsContext {
    pub uid: String,
    pub gids: Vec<String>,
    pub capabilities: Vec<String>,
}

impl VfsContext {
    pub fn new(uid: impl Into<String>) -> Self {
        Self {
            uid: uid.into(),
            gids: Vec::new(),
            capabilities: Vec::new(),
        }
    }
    pub fn with_gids(mut self, gids: Vec<String>) -> Self {
        self.gids = gids;
        self
    }
    pub fn with_capabilities(mut self, caps: Vec<String>) -> Self {
        self.capabilities = caps;
        self
    }
}

/// Walk result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkResult {
    pub qids: Vec<Qid>,
}
