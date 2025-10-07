//! Internal node representation for the in-memory filesystem.

use std::time::SystemTime;

/// Internal filesystem node - either a file or directory
#[derive(Debug, Clone)]
pub(super) enum Node {
    File {
        data: Vec<u8>,
        mtime: SystemTime,
        version: u32,
    },
    Dir {
        mtime: SystemTime,
    },
}

impl Node {
    /// Create a new empty file
    pub fn new_file() -> Self {
        Node::File {
            data: Vec::new(),
            mtime: SystemTime::now(),
            version: 0,
        }
    }

    /// Create a new directory
    pub fn new_dir() -> Self {
        Node::Dir {
            mtime: SystemTime::now(),
        }
    }

    /// Check if this node is a file
    pub fn is_file(&self) -> bool {
        matches!(self, Node::File { .. })
    }

    /// Check if this node is a directory
    pub fn is_dir(&self) -> bool {
        matches!(self, Node::Dir { .. })
    }

    /// Get the modification time
    pub fn mtime(&self) -> SystemTime {
        match self {
            Node::File { mtime, .. } | Node::Dir { mtime } => *mtime,
        }
    }

    /// Get file size (0 for directories)
    pub fn size(&self) -> u64 {
        match self {
            Node::File { data, .. } => data.len() as u64,
            Node::Dir { .. } => 0,
        }
    }

    /// Get file version (0 for directories)
    pub fn version(&self) -> u32 {
        match self {
            Node::File { version, .. } => *version,
            Node::Dir { .. } => 0,
        }
    }
}
