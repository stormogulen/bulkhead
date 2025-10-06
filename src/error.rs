use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error, Serialize, Deserialize)]
pub enum VfsError {
    #[error("not found")]
    NotFound,

    #[error("permission denied")]
    PermissionDenied,

    #[error("already exists")]
    AlreadyExists,

    #[error("invalid: {0}")]
    Invalid(String),

    #[error("backend I/O error: {0}")]
    BackendIo(String),

    // std::io::Error is stringified so it works with Serialize/Deserialize
    #[error("system I/O error: {0}")]
    SystemIo(String),

    #[error("is a directory")]
    IsDirectory,

    #[error("not a directory")]
    NotDirectory,

    #[error("bad offset")]
    BadOffset,

    #[error("partial walk: {0} components")]
    PartialWalk(usize),

    #[error("quota exceeded")]
    QuotaExceeded,

    #[error("path traversal detected")]
    PathTraversal,
}

impl From<std::io::Error> for VfsError {
    fn from(e: std::io::Error) -> Self {
        VfsError::SystemIo(e.to_string())
    }
}

pub type VfsResult<T> = Result<T, VfsError>;
