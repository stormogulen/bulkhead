//! Core VfsMem implementation.

use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use crate::backend::VfsBackend;
use crate::error::{VfsError, VfsResult};
use crate::types::{Dir, File, FileHandle, Qid, ReadOnly, Stat, WalkResult};
use crate::{CanRead, CanWrite};

use super::node::Node;

/// In-memory virtual filesystem backend
#[derive(Debug, Clone)]
pub struct VfsMem {
    nodes: Arc<RwLock<HashMap<String, Node>>>,
    next_fid: Arc<RwLock<u64>>,
}

impl VfsMem {
    /// Create a new in-memory filesystem with a root directory
    pub fn new() -> Self {
        let mut nodes = HashMap::new();
        nodes.insert("/".to_string(), Node::new_dir());

        Self {
            nodes: Arc::new(RwLock::new(nodes)),
            next_fid: Arc::new(RwLock::new(1)),
        }
    }

    /// Generate next unique file ID
    fn next_fid(&self) -> u64 {
        let mut fid = self.next_fid.write().unwrap();
        let id = *fid;
        *fid += 1;
        id
    }

    /// Normalize and validate a path
    pub (super) fn normalize_path(path: &str) -> VfsResult<String> {
        if path.contains("..") {
            return Err(VfsError::InvalidPath(".. traversal not allowed".into()));
        }

        if path.is_empty() {
            return Err(VfsError::InvalidPath("empty path".into()));
        }

        // Handle root
        if path == "/" {
            return Ok("/".to_string());
        }

        // Remove leading/trailing slashes and normalize
        let clean = path.trim_matches('/');
        if clean.is_empty() {
            return Ok("/".to_string());
        }

        // Check for empty components (e.g., "//")
        if clean.split('/').any(|s| s.is_empty()) {
            return Err(VfsError::InvalidPath("empty path component".into()));
        }

        Ok(format!("/{}", clean))
    }

    /// Generate a unique Qid path from a string path
    fn path_to_qid_path(&self, path: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        hasher.finish()
    }

    /// Get immediate children of a directory
    fn get_dir_children(&self, dir_path: &str, nodes: &HashMap<String, Node>) -> Vec<String> {
        let prefix = if dir_path == "/" {
            "/"
        } else {
            &format!("{}/", dir_path)
        };

        nodes
            .keys()
            .filter(|p| p.starts_with(prefix) && *p != dir_path)
            .filter_map(|p| {
                let rest = &p[prefix.len()..];
                // Only immediate children (no nested paths)
                if !rest.contains('/') {
                    Some(rest.to_string())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Convert a Node to a Stat
    fn node_to_stat(&self, path: &str, node: &Node) -> Stat {
        let qid_path = self.path_to_qid_path(path);
        let name = path.split('/').last().unwrap_or(path).to_string();

        let qid = if node.is_file() {
            Qid::new_file(qid_path, node.version())
        } else {
            Qid::new_dir(qid_path, 0)
        };

        Stat {
            qid,
            name,
            size: node.size(),
            mode: if node.is_dir() { 0o755 } else { 0o644 },
            atime: node.mtime(),
            mtime: node.mtime(),
            uid: "user".into(),
            gid: "group".into(),
        }
    }

    /// Ensure parent directory exists
    fn ensure_parent_exists(&self, path: &str, nodes: &HashMap<String, Node>) -> VfsResult<()> {
        if path == "/" {
            return Ok(());
        }

        let parent = path
            .rsplit_once('/')
            .map(|(p, _)| if p.is_empty() { "/" } else { p })
            .unwrap_or("/");

        match nodes.get(parent) {
            Some(node) if node.is_dir() => Ok(()),
            Some(_) => Err(VfsError::NotADirectory(parent.to_string())),
            None => Err(VfsError::NotFound(format!("parent directory: {}", parent))),
        }
    }
}

impl Default for VfsMem {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl VfsBackend for VfsMem {
    async fn walk(&self, start: &str, names: &[String]) -> VfsResult<WalkResult> {
        let start = Self::normalize_path(start)?;
        let nodes = self.nodes.read().unwrap();

        // Verify start exists
        if !nodes.contains_key(&start) {
            return Err(VfsError::NotFound(start));
        }

        let mut current = start;
        let mut qids = Vec::new();

        // Walk each component
        for name in names {
            // Validate name (no slashes, no ..)
            if name.contains('/') || name == ".." {
                return Err(VfsError::InvalidPath(format!("invalid name: {}", name)));
            }

            // Build next path
            let next = if current == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", current, name)
            };

            // Check if it exists
            if let Some(node) = nodes.get(&next) {
                let qid_path = self.path_to_qid_path(&next);
                let qid = if node.is_file() {
                    Qid::new_file(qid_path, node.version())
                } else {
                    Qid::new_dir(qid_path, 0)
                };
                qids.push(qid);
                current = next;
            } else {
                // Partial walk is OK in 9P - return what we have
                break;
            }
        }

        Ok(WalkResult { qids })
    }

    async fn stat(&self, path: &str) -> VfsResult<Stat> {
        let path = Self::normalize_path(path)?;
        let nodes = self.nodes.read().unwrap();

        let node = nodes
            .get(&path)
            .ok_or_else(|| VfsError::NotFound(path.clone()))?;
        Ok(self.node_to_stat(&path, node))
    }

    async fn open<M, T>(&self, path: &str, mode: u32) -> VfsResult<FileHandle<T, M>>
    where
        M: Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let path = Self::normalize_path(path)?;
        let nodes = self.nodes.read().unwrap();

        let node = nodes
            .get(&path)
            .ok_or_else(|| VfsError::NotFound(path.clone()))?;

        // Type check based on T
        let type_name = std::any::type_name::<T>();
        let (qid, is_valid) = if node.is_file() {
            let qid = Qid::new_file(self.path_to_qid_path(&path), node.version());
            (qid, type_name.contains("File"))
        } else {
            let qid = Qid::new_dir(self.path_to_qid_path(&path), 0);
            (qid, type_name.contains("Dir"))
        };

        if !is_valid {
            return if node.is_file() {
                Err(VfsError::NotADirectory(path))
            } else {
                Err(VfsError::IsADirectory(path))
            };
        }

        let fid = self.next_fid();
        Ok(FileHandle::new(fid, qid, path, mode))
    }

    async fn create<M, T>(&self, path: &str, mode: u32) -> VfsResult<FileHandle<T, M>>
    where
        M: Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let path = Self::normalize_path(path)?;

        {
            let mut nodes = self.nodes.write().unwrap();

            // Check if already exists
            if nodes.contains_key(&path) {
                return Err(VfsError::AlreadyExists(path));
            }

            // Ensure parent directory exists
            self.ensure_parent_exists(&path, &nodes)?;

            // Create based on type T
            let type_name = std::any::type_name::<T>();
            if type_name.contains("File") {
                nodes.insert(path.clone(), Node::new_file());
            } else if type_name.contains("Dir") {
                nodes.insert(path.clone(), Node::new_dir());
            } else {
                return Err(VfsError::InvalidArgument("unknown type".into()));
            }
        }

        self.open::<M, T>(&path, mode).await
    }

    async fn read<M: CanRead>(
        &self,
        handle: &FileHandle<File, M>,
        offset: u64,
        count: usize,
    ) -> VfsResult<Vec<u8>> {
        let nodes = self.nodes.read().unwrap();
        let node = nodes
            .get(&handle.path)
            .ok_or_else(|| VfsError::NotFound(handle.path.clone()))?;

        match node {
            Node::File { data, .. } => {
                let start = offset as usize;
                if start > data.len() {
                    return Ok(Vec::new());
                }
                let end = std::cmp::min(start + count, data.len());
                Ok(data[start..end].to_vec())
            }
            Node::Dir { .. } => Err(VfsError::IsADirectory(handle.path.clone())),
        }
    }

    async fn write<M: CanWrite>(
        &self,
        handle: &FileHandle<File, M>,
        offset: u64,
        data: &[u8],
    ) -> VfsResult<usize> {
        let mut nodes = self.nodes.write().unwrap();
        let node = nodes
            .get_mut(&handle.path)
            .ok_or_else(|| VfsError::NotFound(handle.path.clone()))?;

        match node {
            Node::File {
                data: file_data,
                mtime,
                version,
            } => {
                let start = offset as usize;

                // Extend file if necessary
                if start + data.len() > file_data.len() {
                    file_data.resize(start + data.len(), 0);
                }

                file_data[start..start + data.len()].copy_from_slice(data);
                *mtime = SystemTime::now();
                *version += 1;

                Ok(data.len())
            }
            Node::Dir { .. } => Err(VfsError::IsADirectory(handle.path.clone())),
        }
    }

    async fn remove<T>(&self, path: &str) -> VfsResult<()> {
        let path = Self::normalize_path(path)?;

        // Can't remove root
        if path == "/" {
            return Err(VfsError::PermissionDenied("cannot remove root".into()));
        }

        let mut nodes = self.nodes.write().unwrap();

        // Check if it's a directory with children
        if let Some(node) = nodes.get(&path) {
            if node.is_dir() {
                let children = self.get_dir_children(&path, &nodes);
                if !children.is_empty() {
                    return Err(VfsError::InvalidArgument("directory not empty".into()));
                }
            }
        }

        nodes
            .remove(&path)
            .ok_or_else(|| VfsError::NotFound(path))?;

        Ok(())
    }

    async fn readdir(&self, handle: &FileHandle<Dir, ReadOnly>) -> VfsResult<Vec<Stat>> {
        let nodes = self.nodes.read().unwrap();
        let node = nodes
            .get(&handle.path)
            .ok_or_else(|| VfsError::NotFound(handle.path.clone()))?;

        match node {
            Node::Dir { .. } => {
                let children = self.get_dir_children(&handle.path, &nodes);
                let mut stats = Vec::new();

                for child_name in children {
                    let child_path = if handle.path == "/" {
                        format!("/{}", child_name)
                    } else {
                        format!("{}/{}", handle.path, child_name)
                    };

                    if let Some(child_node) = nodes.get(&child_path) {
                        stats.push(self.node_to_stat(&child_path, child_node));
                    }
                }

                Ok(stats)
            }
            Node::File { .. } => Err(VfsError::NotADirectory(handle.path.clone())),
        }
    }
}
