use std::{
    env,
    error::Error,
    net::SocketAddr,
    path::PathBuf,
};

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use filestorage_core::{FileStorage, StorageError};
use serde::Serialize;

type AnyError = Box<dyn Error + Send + Sync>;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("storage node failed: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), AnyError> {
    let settings = Settings::from_env()?;
    let storage = FileStorage::new(&settings.storage_root).await?;
    let state = AppState { storage };
    let router = build_router(state);

    let listener = tokio::net::TcpListener::bind(settings.bind_address).await?;
    println!(
        "listening on http://{} (storage root: {})",
        settings.bind_address,
        settings.storage_root.display()
    );
    axum::serve(listener, router).await?;
    Ok(())
}

#[derive(Clone)]
struct AppState {
    storage: FileStorage,
}

fn build_router(state: AppState) -> Router {
    Router::new()
        .route(
            "/objects/*key",
            get(get_object).put(put_object).delete(delete_object),
        )
        .with_state(state)
}

async fn put_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
    body: Bytes,
) -> Result<impl IntoResponse, ApiError> {
    ensure_key_present(&key)?;
    state.storage.put(&key, &body).await?;
    Ok(StatusCode::CREATED)
}

async fn get_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Response, ApiError> {
    ensure_key_present(&key)?;
    let bytes = state.storage.get(&key).await?;
    let len = bytes.len();

    let mut response = Response::new(bytes.into());
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream"));
    response.headers_mut().insert(
        header::CONTENT_LENGTH,
        HeaderValue::from_str(&len.to_string()).expect("content length header"),
    );
    Ok(response)
}

async fn delete_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    ensure_key_present(&key)?;
    state.storage.delete(&key).await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

#[derive(Debug)]
enum ApiError {
    BadRequest(String),
    NotFound(String),
    Internal(String),
}

impl ApiError {
    fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

impl From<StorageError> for ApiError {
    fn from(value: StorageError) -> Self {
        match value {
            StorageError::InvalidKey(msg) => Self::BadRequest(msg),
            StorageError::NotFound(key) => Self::NotFound(key),
            StorageError::Io(err) => Self::internal(format!("storage I/O error: {err}")),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        match self {
            ApiError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, Json(ErrorBody { error: msg })).into_response()
            }
            ApiError::NotFound(key) => (
                StatusCode::NOT_FOUND,
                Json(ErrorBody {
                    error: format!("object `{key}` not found"),
                }),
            )
                .into_response(),
            ApiError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody { error: msg }),
            )
                .into_response(),
        }
    }
}

fn ensure_key_present(key: &str) -> Result<(), ApiError> {
    if key.is_empty() {
        return Err(ApiError::bad_request("object key cannot be empty"));
    }
    Ok(())
}

#[derive(Debug)]
struct Settings {
    bind_address: SocketAddr,
    storage_root: PathBuf,
}

impl Settings {
    fn from_env() -> Result<Self, AnyError> {
        let bind_address = env::var("FILESTORAGE_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:8080".to_string())
            .parse()?;
        let storage_root = PathBuf::from(
            env::var("FILESTORAGE_DATA_DIR").unwrap_or_else(|_| "data".to_string()),
        );
        Ok(Self {
            bind_address,
            storage_root,
        })
    }
}
