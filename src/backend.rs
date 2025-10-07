use crate::error::VfsResult;
use crate::CanRead;
use crate::CanWrite;
use crate::Dir;
use crate::File;
use crate::FileHandle;
use crate::ReadOnly;
use crate::Stat;
use crate::WalkResult;

/// Core trait implemented by all backends.
/// Note: not `dyn`-compatible because of generic methods.
#[async_trait::async_trait]
pub trait VfsBackend: Send + Sync + 'static {
    async fn walk(&self, start: &str, names: &[String]) -> VfsResult<WalkResult>;

    async fn stat(&self, path: &str) -> VfsResult<Stat>;

    async fn open<M, T>(&self, path: &str, mode: u32) -> VfsResult<FileHandle<T, M>>
    where
        M: Send + Sync + 'static,
        T: Send + Sync + 'static;

    async fn create<M, T>(&self, path: &str, mode: u32) -> VfsResult<FileHandle<T, M>>
    where
        M: Send + Sync + 'static,
        T: Send + Sync + 'static;

    async fn read<M: CanRead>(
        &self,
        handle: &FileHandle<File, M>,
        offset: u64,
        count: usize,
    ) -> VfsResult<Vec<u8>>;

    async fn write<M: CanWrite>(
        &self,
        handle: &FileHandle<File, M>,
        offset: u64,
        data: &[u8],
    ) -> VfsResult<usize>;

    async fn remove<T>(&self, path: &str) -> VfsResult<()>;

    async fn readdir(&self, handle: &FileHandle<Dir, ReadOnly>) -> VfsResult<Vec<Stat>>;
}
