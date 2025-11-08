use filestorage_core::{FileStorage, StorageError};
use tempfile::tempdir;

#[tokio::test]
async fn put_get_delete_round_trip() {
    let tmp = tempdir().unwrap();
    let storage = FileStorage::new(tmp.path()).await.unwrap();

    storage.put("sample.txt", b"hello").await.unwrap();
    let bytes = storage.get("sample.txt").await.unwrap();
    assert_eq!(bytes, b"hello");

    storage.delete("sample.txt").await.unwrap();
    let err = storage.get("sample.txt").await.unwrap_err();
    assert!(matches!(err, StorageError::NotFound(key) if key == "sample.txt"));
}

#[tokio::test]
async fn rejects_keys_with_parent_dirs() {
    let tmp = tempdir().unwrap();
    let storage = FileStorage::new(tmp.path()).await.unwrap();
    let err = storage.put("../bad", b"nope").await.unwrap_err();
    assert!(matches!(err, StorageError::InvalidKey(_)));
}
