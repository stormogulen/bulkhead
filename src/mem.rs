//! In-memory VFS backend implementation.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use crate::backend::VfsBackend;
use crate::error::{VfsError, VfsResult};
use crate::types::{Dir, File, FileHandle, Qid, ReadOnly, ReadWrite, Stat, WalkResult};
use crate::{CanRead, CanWrite};

#[derive(Debug, Clone)]
pub struct VfsMem {
    files: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    next_fid: Arc<RwLock<u64>>,
}

impl VfsMem {
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
            next_fid: Arc::new(RwLock::new(1)),
        }
    }

    fn next_fid(&self) -> u64 {
        let mut fid = self.next_fid.write().unwrap();
        let id = *fid;
        *fid += 1;
        id
    }

    fn now() -> SystemTime {
        SystemTime::now()
    }
}

#[async_trait::async_trait]
impl VfsBackend for VfsMem {
    async fn walk(&self, _start: &str, names: &[String]) -> VfsResult<WalkResult> {
        let qids = names.iter().map(|_| Qid::new_file(0, 0)).collect();
        Ok(WalkResult { qids })
    }

    async fn stat(&self, path: &str) -> VfsResult<Stat> {
        let files = self.files.read().unwrap();
        if let Some(data) = files.get(path) {
            Ok(Stat {
                qid: Qid::new_file(0, 0),
                name: path.to_string(),
                size: data.len() as u64,
                mode: 0o644,
                atime: Self::now(),
                mtime: Self::now(),
                uid: "user".into(),
                gid: "group".into(),
            })
        } else {
            Err(VfsError::NotFound)
        }
    }

    async fn open<M, T>(&self, path: &str, _mode: u32) -> VfsResult<FileHandle<T, M>>
    where
        M: Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        let fid = self.next_fid();
        let qid = Qid::new_file(0, 0);
        Ok(FileHandle::new(fid, qid, path.to_string(), 0))
    }

    async fn create<M, T>(&self, path: &str, _mode: u32) -> VfsResult<FileHandle<T, M>>
    where
        M: Send + Sync + 'static,
        T: Send + Sync + 'static,
    {
        {
            // Insert file while holding write lock
            let mut files = self.files.write().unwrap();
            files.insert(path.to_string(), Vec::new());
        } // lock is dropped here

        // Now safe to await
        self.open::<M, T>(path, 0).await
    }

    async fn read<M: CanRead>(
        &self,
        handle: &FileHandle<File, M>,
        offset: u64,
        count: usize,
    ) -> VfsResult<Vec<u8>> {
        let files = self.files.read().unwrap();
        let data = files.get(&handle.path).ok_or(VfsError::NotFound)?;
        let start = offset as usize;
        let end = std::cmp::min(start + count, data.len());
        Ok(data[start..end].to_vec())
    }

    async fn write<M: CanWrite>(
        &self,
        handle: &FileHandle<File, M>,
        offset: u64,
        data: &[u8],
    ) -> VfsResult<usize> {
        let mut files = self.files.write().unwrap();
        let entry = files.get_mut(&handle.path).ok_or(VfsError::NotFound)?;
        let start = offset as usize;
        if start > entry.len() {
            return Err(VfsError::BadOffset);
        }
        if start + data.len() > entry.len() {
            entry.resize(start + data.len(), 0);
        }
        entry[start..start + data.len()].copy_from_slice(data);
        Ok(data.len())
    }

    async fn remove<T>(&self, path: &str) -> VfsResult<()> {
        let mut files = self.files.write().unwrap();
        if files.remove(path).is_none() {
            return Err(VfsError::NotFound);
        }
        Ok(())
    }

    async fn readdir(&self, _handle: &FileHandle<Dir, ReadOnly>) -> VfsResult<Vec<Stat>> {
        let files = self.files.read().unwrap();
        let mut stats = Vec::new();
        for path in files.keys() {
            stats.push(Stat {
                qid: Qid::new_file(0, 0),
                name: path.clone(),
                size: files.get(path).unwrap().len() as u64,
                mode: 0o644,
                atime: Self::now(),
                mtime: Self::now(),
                uid: "user".into(),
                gid: "group".into(),
            });
        }
        Ok(stats)
    }
}
