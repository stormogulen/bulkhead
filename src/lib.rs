//! vfs-core: Core traits and types for a Unix-like, 9P-friendly VFS with zero-trust principles.

pub mod backend;
pub mod error;
pub mod types;

// Re-export main items for convenience
pub use backend::*;
pub use error::{VfsError, VfsResult};
pub use types::*;
