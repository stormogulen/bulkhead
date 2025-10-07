// error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VfsError {
    #[error("path not found: {0}")]
    NotFound(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("already exists: {0}")]
    AlreadyExists(String),

    #[error("not a directory: {0}")]
    NotADirectory(String),

    #[error("is a directory: {0}")]
    IsADirectory(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("invalid path: {0}")]
    InvalidPath(String),

    #[error("invalid offset")]
    BadOffset,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("lock poisoned")]
    LockPoisoned,
}

pub type VfsResult<T> = Result<T, VfsError>;

impl<T> From<std::sync::PoisonError<T>> for VfsError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        VfsError::LockPoisoned
    }
}

// use serde::{Deserialize, Serialize};
// use thiserror::Error;

// #[derive(Debug, Error, Serialize, Deserialize)]
// pub enum VfsError {
//     #[error("not found")]
//     NotFound,

//     #[error("permission denied")]
//     PermissionDenied,

//     #[error("already exists")]
//     AlreadyExists,

//     #[error("invalid: {0}")]
//     Invalid(String),

//     #[error("backend I/O error: {0}")]
//     BackendIo(String),

//     // std::io::Error is stringified so it works with Serialize/Deserialize
//     #[error("system I/O error: {0}")]
//     SystemIo(String),

//     #[error("is a directory")]
//     IsDirectory,

//     #[error("not a directory")]
//     NotDirectory,

//     #[error("bad offset")]
//     BadOffset,

//     #[error("partial walk: {0} components")]
//     PartialWalk(usize),

//     #[error("quota exceeded")]
//     QuotaExceeded,

//     #[error("path traversal detected")]
//     PathTraversal,
// }

// impl From<std::io::Error> for VfsError {
//     fn from(e: std::io::Error) -> Self {
//         VfsError::SystemIo(e.to_string())
//     }
// }

// pub type VfsResult<T> = Result<T, VfsError>;
