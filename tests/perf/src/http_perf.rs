use axum::{
    body::Bytes,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use filestorage_core::FileStorage;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tempfile::tempdir;
use tokio::task::JoinSet;

#[derive(Clone)]
struct AppState {
    storage: FileStorage,
}

fn build_test_router(state: AppState) -> Router {
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
) -> Result<impl IntoResponse, StatusCode> {
    state.storage.put(&key, &body).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::CREATED)
}

async fn get_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Bytes, StatusCode> {
    state.storage.get(&key).await
        .map(Bytes::from)
        .map_err(|_| StatusCode::NOT_FOUND)
}

async fn delete_object(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    state.storage.delete(&key).await.map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(StatusCode::NO_CONTENT)
}

/// Start a test server in the background
async fn start_test_server() -> (String, tokio::task::JoinHandle<()>) {
    let tmp = tempdir().unwrap();
    let storage = FileStorage::new(tmp.path()).await.unwrap();
    let state = AppState { storage };
    let router = build_test_router(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 0)); // Random port
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let bound_addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", bound_addr);

    let handle = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    (base_url, handle)
}

#[tokio::test]
async fn http_put_latency() {
    let (base_url, _server) = start_test_server().await;
    let client = reqwest::Client::new();

    let sizes = vec![
        ("1KB", 1024),
        ("10KB", 10 * 1024),
        ("100KB", 100 * 1024),
        ("1MB", 1024 * 1024),
    ];

    println!("\n=== HTTP PUT Latency ===");

    for (name, size) in sizes {
        let data = vec![0xAB; size];
        let url = format!("{}/objects/test-{}", base_url, name);

        let start = Instant::now();
        let response = client.put(&url)
            .body(data)
            .send()
            .await
            .unwrap();
        let elapsed = start.elapsed();

        assert_eq!(response.status(), StatusCode::CREATED);

        println!("{}: {:?} ({:.2} MB/s)",
                 name,
                 elapsed,
                 size as f64 / (1024.0 * 1024.0) / elapsed.as_secs_f64());
    }
}

#[tokio::test]
async fn http_get_latency() {
    let (base_url, _server) = start_test_server().await;
    let client = reqwest::Client::new();

    let sizes = vec![
        ("1KB", 1024),
        ("10KB", 10 * 1024),
        ("100KB", 100 * 1024),
        ("1MB", 1024 * 1024),
    ];

    println!("\n=== HTTP GET Latency ===");

    for (name, size) in &sizes {
        let data = vec![0xAB; *size];
        let url = format!("{}/objects/test-{}", base_url, name);

        // Pre-populate
        client.put(&url).body(data).send().await.unwrap();

        // Measure GET
        let start = Instant::now();
        let response = client.get(&url).send().await.unwrap();
        let bytes = response.bytes().await.unwrap();
        let elapsed = start.elapsed();

        assert_eq!(bytes.len(), *size);

        println!("{}: {:?} ({:.2} MB/s)",
                 name,
                 elapsed,
                 *size as f64 / (1024.0 * 1024.0) / elapsed.as_secs_f64());
    }
}

#[tokio::test]
async fn http_delete_latency() {
    let (base_url, _server) = start_test_server().await;
    let client = reqwest::Client::new();
    let data = vec![0xAB; 1024];

    println!("\n=== HTTP DELETE Latency ===");

    let mut total = std::time::Duration::ZERO;
    let iterations = 10;

    for i in 0..iterations {
        let url = format!("{}/objects/delete-test-{}", base_url, i);

        // Pre-populate
        client.put(&url).body(data.clone()).send().await.unwrap();

        // Measure DELETE
        let start = Instant::now();
        let response = client.delete(&url).send().await.unwrap();
        let elapsed = start.elapsed();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        total += elapsed;
    }

    let avg = total / iterations;
    println!("Average: {:?}", avg);
}

#[tokio::test]
async fn http_concurrent_requests() {
    let (base_url, _server) = start_test_server().await;
    let client = Arc::new(reqwest::Client::new());
    let num_requests = 50;
    let data = vec![0xAB; 10 * 1024]; // 10KB per request

    println!("\n=== HTTP Concurrent Requests ===");

    let start = Instant::now();
    let mut tasks = JoinSet::new();

    for i in 0..num_requests {
        let client = Arc::clone(&client);
        let base_url = base_url.clone();
        let data = data.clone();

        tasks.spawn(async move {
            let url = format!("{}/objects/concurrent-{}", base_url, i);
            client.put(&url).body(data).send().await.unwrap()
        });
    }

    while tasks.join_next().await.is_some() {}

    let elapsed = start.elapsed();
    let ops_per_sec = num_requests as f64 / elapsed.as_secs_f64();

    println!("Requests: {}", num_requests);
    println!("Total time: {:?}", elapsed);
    println!("Throughput: {:.2} req/sec", ops_per_sec);
    println!("Avg latency: {:.2}ms", elapsed.as_millis() as f64 / num_requests as f64);
}

#[tokio::test]
async fn http_mixed_concurrent_workload() {
    let (base_url, _server) = start_test_server().await;
    let client = Arc::new(reqwest::Client::new());
    let data = vec![0xAB; 10 * 1024]; // 10KB

    // Pre-populate some objects
    for i in 0..20 {
        let url = format!("{}/objects/existing-{}", base_url, i);
        client.put(&url).body(data.clone()).send().await.unwrap();
    }

    let num_requests = 60;
    println!("\n=== HTTP Mixed Concurrent Workload ===");

    let start = Instant::now();
    let mut tasks = JoinSet::new();

    for i in 0..num_requests {
        let client = Arc::clone(&client);
        let base_url = base_url.clone();
        let data = data.clone();

        tasks.spawn(async move {
            let url = match i % 3 {
                0 => {
                    // PUT (33%)
                    let url = format!("{}/objects/new-{}", base_url, i);
                    client.put(&url).body(data).send().await.unwrap();
                    url
                }
                1 => {
                    // GET (33%)
                    let url = format!("{}/objects/existing-{}", base_url, i % 20);
                    client.get(&url).send().await.unwrap();
                    url
                }
                _ => {
                    // DELETE (33%)
                    let url = format!("{}/objects/existing-{}", base_url, i % 20);
                    client.delete(&url).send().await.unwrap();
                    url
                }
            };
            url
        });
    }

    while tasks.join_next().await.is_some() {}

    let elapsed = start.elapsed();
    let ops_per_sec = num_requests as f64 / elapsed.as_secs_f64();

    println!("Requests: {}", num_requests);
    println!("Mix: 33% PUT, 33% GET, 33% DELETE");
    println!("Total time: {:?}", elapsed);
    println!("Throughput: {:.2} req/sec", ops_per_sec);
}

#[tokio::test]
async fn http_overhead_comparison() {
    let (base_url, _server) = start_test_server().await;
    let http_client = reqwest::Client::new();

    // Also create direct storage for comparison
    let tmp = tempdir().unwrap();
    let direct_storage = FileStorage::new(tmp.path()).await.unwrap();

    let data = vec![0xAB; 100 * 1024]; // 100KB
    let iterations = 10;

    println!("\n=== HTTP Overhead Comparison ===");

    // Measure HTTP PUT
    let start = Instant::now();
    for i in 0..iterations {
        let url = format!("{}/objects/http-test-{}", base_url, i);
        http_client.put(&url).body(data.clone()).send().await.unwrap();
    }
    let http_time = start.elapsed();

    // Measure direct storage PUT
    let start = Instant::now();
    for i in 0..iterations {
        let key = format!("direct-test-{}", i);
        direct_storage.put(&key, &data).await.unwrap();
    }
    let direct_time = start.elapsed();

    let overhead = http_time.as_micros() as f64 / direct_time.as_micros() as f64;

    println!("HTTP PUT ({}x): {:?} ({:.2}ms avg)",
             iterations, http_time, http_time.as_millis() as f64 / iterations as f64);
    println!("Direct PUT ({}x): {:?} ({:.2}ms avg)",
             iterations, direct_time, direct_time.as_millis() as f64 / iterations as f64);
    println!("HTTP overhead: {:.2}x slower", overhead);
}
