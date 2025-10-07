use bulkhead::*;
use crate::{ VfsError};
use bulkhead::backends::VfsMem;

use bulkhead::backend::VfsBackend;
use bulkhead::types::{File, Dir, ReadOnly, ReadWrite};

//use tokio::runtime::Runtime;

#[tokio::test]
async fn test_vfs_mem_basic_operations() -> VfsResult<()> {
    // Create a new in-memory backend
    let backend = VfsMem::new();

    // --- Create a file ---
    let fh: FileHandle<File, ReadWrite> = backend.create::<ReadWrite, File>("file1", 0).await?;
    assert_eq!((fh.path), "/file1");

    // --- Write to file ---
    let content = b"Hello VFS!";
    let written = backend.write(&fh, 0, content).await?;
    assert_eq!(written, content.len());

    // --- Read from file ---
    let read_data = backend.read(&fh, 0, content.len()).await?;
    assert_eq!(read_data, content);

    // --- Stat the file ---
    let stat: Stat = backend.stat("file1").await?;
    assert_eq!(stat.name, "file1");
    assert_eq!(stat.size, content.len() as u64);

    // --- Remove the file ---
    backend.remove::<File>("file1").await?;
    let result = backend.stat("file1").await;
    println!("stat result: {:?}", result);
    assert!(matches!(result, Err(VfsError::NotFound(_))));

    Ok(())
}

#[tokio::test]
async fn test_vfs_mem_readdir() -> VfsResult<()> {
    let backend = VfsMem::new();

    // Create files
    backend.create::<ReadWrite, File>("file1", 0).await?;
    backend.create::<ReadWrite, File>("file2", 0).await?;

    // Create a directory handle (fake, path irrelevant for VfsMem)
    //let dir_handle = FileHandle::<Dir, ReadOnly>::new(0, Qid::new_dir(0, 0), "/", 0);
    let dir_handle = FileHandle::<Dir, ReadOnly>::new(0, Qid::new_dir(0, 0), "/".to_string(), 0);

    let entries = backend.readdir(&dir_handle).await?;
    let names: Vec<_> = entries.iter().map(|s| s.name.as_str()).collect();

    assert!(names.contains(&"file1"));
    assert!(names.contains(&"file2"));

    Ok(())
}

#[tokio::test]
async fn test_vfs_mem_walk() -> VfsResult<()> {
    let backend = VfsMem::new();

    // create /file1 as a directory
    backend.create::<ReadWrite, Dir>("/file1", 0).await?;
    // create /file1/file2 as a file
    backend.create::<ReadWrite, File>("/file1/file2", 0).await?;

    let walk_res = backend
        .walk("/", &["file1".to_string(), "file2".to_string()])
        .await?;
    assert_eq!(walk_res.qids.len(), 2);

    Ok(())
}

