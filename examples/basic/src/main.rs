use filestorage_core::FileStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = FileStorage::new("example-data").await?;
    storage
        .put("hello.txt", b"Hello from the example crate!")
        .await?;

    let contents = storage.get("hello.txt").await?;
    println!("{}", String::from_utf8_lossy(&contents));
    storage.delete("hello.txt").await?;
    Ok(())
}
