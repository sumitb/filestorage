use std::{
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

use thiserror::Error;
use tokio::fs;

#[derive(Clone, Debug)]
pub struct FileStorage {
    root: PathBuf,
}

impl FileStorage {
    pub async fn new<P: AsRef<Path>>(root: P) -> Result<Self, StorageError> {
        let root = root.as_ref().to_path_buf();
        fs::create_dir_all(&root).await?;
        Ok(Self { root })
    }

    pub async fn put(&self, key: &str, data: &[u8]) -> Result<(), StorageError> {
        let path = self.path_for(key)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, data).await.map_err(StorageError::from)?;
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Vec<u8>, StorageError> {
        let path = self.path_for(key)?;
        match fs::read(path).await {
            Ok(bytes) => Ok(bytes),
            Err(err) if err.kind() == ErrorKind::NotFound => {
                Err(StorageError::NotFound(key.to_string()))
            }
            Err(err) => Err(StorageError::from(err)),
        }
    }

    pub async fn delete(&self, key: &str) -> Result<(), StorageError> {
        let path = self.path_for(key)?;
        match fs::remove_file(path).await {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == ErrorKind::NotFound => {
                Err(StorageError::NotFound(key.to_string()))
            }
            Err(err) => Err(StorageError::from(err)),
        }
    }

    fn path_for(&self, key: &str) -> Result<PathBuf, StorageError> {
        validate_key(key)?;
        Ok(self.root.join(key))
    }
}

fn validate_key(key: &str) -> Result<(), StorageError> {
    if key.is_empty() {
        return Err(StorageError::InvalidKey("key cannot be empty".to_string()));
    }

    let path = Path::new(key);
    if path.is_absolute() {
        return Err(StorageError::InvalidKey(format!(
            "absolute paths are not allowed (got `{key}`)"
        )));
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => continue,
            _ => {
                return Err(StorageError::InvalidKey(format!(
                    "`{key}` contains unsupported segments"
                )));
            }
        }
    }

    Ok(())
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("invalid object key: {0}")]
    InvalidKey(String),
    #[error("object not found: {0}")]
    NotFound(String),
    #[error("storage I/O error: {0}")]
    Io(#[from] std::io::Error),
}
