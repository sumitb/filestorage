#[cfg(test)]
mod tests {
    use filestorage_core::FileStorage;
    use tempfile::tempdir;

    #[tokio::test]
    async fn writes_and_reads_objects() {
        let tmp = tempdir().unwrap();
        let storage = FileStorage::new(tmp.path()).await.unwrap();

        storage.put("smoke.txt", b"smoke").await.unwrap();
        let bytes = storage.get("smoke.txt").await.unwrap();
        assert_eq!(bytes, b"smoke");
    }
}
