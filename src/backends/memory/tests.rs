//! Tests for the in-memory VFS backend.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{File, Dir, ReadWrite, WriteOnly, ReadOnly};
    use crate::backends::VfsMem;
    use crate::backend::VfsBackend;
    use crate::VfsError;
   
    #[tokio::test]
    async fn test_normalize_path() {
        assert_eq!(VfsMem::normalize_path("/").unwrap(), "/");
        assert_eq!(VfsMem::normalize_path("/foo").unwrap(), "/foo");
        assert_eq!(VfsMem::normalize_path("foo").unwrap(), "/foo");
        assert_eq!(VfsMem::normalize_path("/foo/bar/").unwrap(), "/foo/bar");

        assert!(VfsMem::normalize_path("..").is_err());
        assert!(VfsMem::normalize_path("/foo/../bar").is_err());
        assert!(VfsMem::normalize_path("").is_err());
    }

    #[tokio::test]
    async fn test_create_and_stat() {
        let vfs = VfsMem::new();

        // Create a file
        let _handle = vfs
            .create::<WriteOnly, File>("/test.txt", 0o644)
            .await
            .unwrap();

        // Stat it
        let stat = vfs.stat("/test.txt").await.unwrap();
        assert_eq!(stat.name, "test.txt");
        assert_eq!(stat.size, 0);
    }

    #[tokio::test]
    async fn test_write_and_read() {
        let vfs = VfsMem::new();

        let handle = vfs
            .create::<ReadWrite, File>("/test.txt", 0o644)
            .await
            .unwrap();

        // Write data
        let written = vfs.write(&handle, 0, b"Hello, World!").await.unwrap();
        assert_eq!(written, 13);

        // Read it back
        let data = vfs.read(&handle, 0, 100).await.unwrap();
        assert_eq!(data, b"Hello, World!");
    }

    #[tokio::test]
    async fn test_partial_write() {
        let vfs = VfsMem::new();
        let handle = vfs
            .create::<ReadWrite, File>("/test.txt", 0o644)
            .await
            .unwrap();

        // Write at different offsets
        vfs.write(&handle, 0, b"Hello").await.unwrap();
        vfs.write(&handle, 7, b"World").await.unwrap();

        let data = vfs.read(&handle, 0, 100).await.unwrap();
        // Note: gap at offset 5-6 will be filled with zeros
        assert_eq!(data.len(), 12);
    }

    #[tokio::test]
    async fn test_directory_operations() {
        let vfs = VfsMem::new();

        // Create directories
        vfs.create::<ReadOnly, Dir>("/dir1", 0o755).await.unwrap();
        vfs.create::<ReadOnly, Dir>("/dir1/dir2", 0o755)
            .await
            .unwrap();

        // Create file in nested dir
        vfs.create::<WriteOnly, File>("/dir1/dir2/file.txt", 0o644)
            .await
            .unwrap();

        // List root
        let root_handle = vfs.open::<ReadOnly, Dir>("/", 0).await.unwrap();
        let entries = vfs.readdir(&root_handle).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "dir1");

        // List nested dir
        let dir2_handle = vfs.open::<ReadOnly, Dir>("/dir1/dir2", 0).await.unwrap();
        let entries = vfs.readdir(&dir2_handle).await.unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "file.txt");
    }

    #[tokio::test]
    async fn test_walk() {
        let vfs = VfsMem::new();

        vfs.create::<ReadOnly, Dir>("/a", 0o755).await.unwrap();
        vfs.create::<ReadOnly, Dir>("/a/b", 0o755).await.unwrap();
        vfs.create::<WriteOnly, File>("/a/b/c.txt", 0o644)
            .await
            .unwrap();

        let result = vfs
            .walk("/", &["a".into(), "b".into(), "c.txt".into()])
            .await
            .unwrap();
        assert_eq!(result.qids.len(), 3);
    }

    #[tokio::test]
    async fn test_remove() {
        let vfs = VfsMem::new();

        vfs.create::<WriteOnly, File>("/test.txt", 0o644)
            .await
            .unwrap();
        assert!(vfs.stat("/test.txt").await.is_ok());

        vfs.remove::<File>("/test.txt").await.unwrap();
        assert!(vfs.stat("/test.txt").await.is_err());
    }

    #[tokio::test]
    async fn test_cannot_remove_nonempty_dir() {
        let vfs = VfsMem::new();

        vfs.create::<ReadOnly, Dir>("/dir", 0o755).await.unwrap();
        vfs.create::<WriteOnly, File>("/dir/file.txt", 0o644)
            .await
            .unwrap();

        let result = vfs.remove::<Dir>("/dir").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_type_mismatch() {
        let vfs = VfsMem::new();

        vfs.create::<ReadOnly, Dir>("/dir", 0o755).await.unwrap();

        // Try to open directory as file
        let result = vfs.open::<ReadOnly, File>("/dir", 0).await;
        assert!(matches!(result, Err(VfsError::IsADirectory(_))));
    }

    #[tokio::test]
    async fn test_path_traversal_blocked() {
        let vfs = VfsMem::new();

        let result = vfs.create::<WriteOnly, File>("/../etc/passwd", 0o644).await;
        assert!(matches!(result, Err(VfsError::InvalidPath(_))));
    }
}
