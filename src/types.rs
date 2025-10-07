// types.rs
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::time::SystemTime;

/// Object types
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct File;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Dir;

/// Access modes
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReadOnly;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct WriteOnly;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReadWrite;

/// Traits for compile-time mode checking
pub trait CanRead: Send + Sync {}
impl CanRead for ReadOnly {}
impl CanRead for ReadWrite {}

pub trait CanWrite: Send + Sync {}
impl CanWrite for WriteOnly {}
impl CanWrite for ReadWrite {}

/// Unique file identifier (like 9P `qid`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Qid<T = ()> {
    pub ty: u8,
    pub version: u32,
    pub path: u64,
    #[serde(skip)]
    pub _marker: PhantomData<T>,
}

impl<T> Qid<T> {
    /// Create a new Qid for a file (ty = 0x00)
    pub fn new_file(path: u64, version: u32) -> Self {
        Self {
            ty: 0x00,
            version,
            path,
            _marker: PhantomData,
        }
    }

    /// Create a new Qid for a directory (ty = 0x80)
    pub fn new_dir(path: u64, version: u32) -> Self {
        Self {
            ty: 0x80,
            version,
            path,
            _marker: PhantomData,
        }
    }
}

/// Result of a walk operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkResult {
    pub qids: Vec<Qid>,
}

/// File metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stat<T = ()> {
    pub qid: Qid<T>,
    pub name: String,
    pub size: u64,
    pub mode: u32,
    pub atime: SystemTime,
    pub mtime: SystemTime,
    pub uid: String,
    pub gid: String,
}

/// Open file/directory handle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHandle<T = (), M = ()> {
    pub fid: u64,
    pub qid: Qid<T>,
    pub path: String,
    pub mode: u32,
    #[serde(skip)]
    pub _marker: PhantomData<(T, M)>,
}

impl<T, M> FileHandle<T, M> {
    pub fn new(fid: u64, qid: Qid<T>, path: String, mode: u32) -> Self {
        Self {
            fid,
            qid,
            path,
            mode,
            _marker: PhantomData,
        }
    }
}
