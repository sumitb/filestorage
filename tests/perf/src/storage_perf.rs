use filestorage_core::FileStorage;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempfile::tempdir;
use tokio::task::JoinSet;

/// Test concurrent writes to different keys
#[tokio::test]
async fn concurrent_writes_different_keys() {
    let tmp = tempdir().unwrap();
    let storage = Arc::new(FileStorage::new(tmp.path()).await.unwrap());
    let num_tasks = 100;
    let data = vec![0xAB; 1024]; // 1KB per object

    let start = Instant::now();
    let mut tasks = JoinSet::new();

    for i in 0..num_tasks {
        let storage = Arc::clone(&storage);
        let data = data.clone();
        tasks.spawn(async move {
            let key = format!("object-{}", i);
            storage.put(&key, &data).await.unwrap();
        });
    }

    // Wait for all tasks
    while tasks.join_next().await.is_some() {}

    let elapsed = start.elapsed();
    let ops_per_sec = num_tasks as f64 / elapsed.as_secs_f64();

    println!("\n=== Concurrent Writes (Different Keys) ===");
    println!("Tasks: {}", num_tasks);
    println!("Total time: {:?}", elapsed);
    println!("Throughput: {:.2} ops/sec", ops_per_sec);
    println!("Avg latency: {:.2}ms", elapsed.as_millis() as f64 / num_tasks as f64);
}

/// Test concurrent reads of the same key
#[tokio::test]
async fn concurrent_reads_same_key() {
    let tmp = tempdir().unwrap();
    let storage = Arc::new(FileStorage::new(tmp.path()).await.unwrap());
    let data = vec![0xAB; 100 * 1024]; // 100KB object

    // Pre-populate
    storage.put("shared-object", &data).await.unwrap();

    let num_tasks = 100;
    let start = Instant::now();
    let mut tasks = JoinSet::new();

    for _ in 0..num_tasks {
        let storage = Arc::clone(&storage);
        tasks.spawn(async move {
            let bytes = storage.get("shared-object").await.unwrap();
            assert_eq!(bytes.len(), 100 * 1024);
        });
    }

    while tasks.join_next().await.is_some() {}

    let elapsed = start.elapsed();
    let ops_per_sec = num_tasks as f64 / elapsed.as_secs_f64();

    println!("\n=== Concurrent Reads (Same Key) ===");
    println!("Tasks: {}", num_tasks);
    println!("Object size: 100KB");
    println!("Total time: {:?}", elapsed);
    println!("Throughput: {:.2} ops/sec", ops_per_sec);
    println!("Avg latency: {:.2}ms", elapsed.as_millis() as f64 / num_tasks as f64);
}

/// Test mixed workload: reads, writes, deletes
#[tokio::test]
async fn mixed_workload() {
    let tmp = tempdir().unwrap();
    let storage = Arc::new(FileStorage::new(tmp.path()).await.unwrap());
    let data = vec![0xAB; 10 * 1024]; // 10KB per object

    // Pre-populate some objects for reading/deleting
    for i in 0..50 {
        let key = format!("existing-{}", i);
        storage.put(&key, &data).await.unwrap();
    }

    let num_tasks = 100;
    let start = Instant::now();
    let mut tasks = JoinSet::new();

    for i in 0..num_tasks {
        let storage = Arc::clone(&storage);
        let data = data.clone();

        tasks.spawn(async move {
            match i % 10 {
                // 50% writes (0-4)
                0..=4 => {
                    let key = format!("new-{}", i);
                    storage.put(&key, &data).await.unwrap();
                }
                // 30% reads (5-7)
                5..=7 => {
                    let key = format!("existing-{}", i % 50);
                    let _ = storage.get(&key).await;
                }
                // 20% deletes (8-9)
                _ => {
                    let key = format!("existing-{}", i % 50);
                    let _ = storage.delete(&key).await;
                }
            }
        });
    }

    while tasks.join_next().await.is_some() {}

    let elapsed = start.elapsed();
    let ops_per_sec = num_tasks as f64 / elapsed.as_secs_f64();

    println!("\n=== Mixed Workload ===");
    println!("Tasks: {}", num_tasks);
    println!("Mix: 50% writes, 30% reads, 20% deletes");
    println!("Total time: {:?}", elapsed);
    println!("Throughput: {:.2} ops/sec", ops_per_sec);
}

/// Test write contention - multiple tasks writing to same key
/// This tests the race condition behavior
#[tokio::test]
async fn write_contention_same_key() {
    let tmp = tempdir().unwrap();
    let storage = Arc::new(FileStorage::new(tmp.path()).await.unwrap());
    let num_tasks = 50;

    let start = Instant::now();
    let mut tasks = JoinSet::new();

    for i in 0..num_tasks {
        let storage = Arc::clone(&storage);
        tasks.spawn(async move {
            // Each task writes different data to the same key
            let data = vec![i as u8; 1024];
            storage.put("contended-key", &data).await.unwrap();
        });
    }

    while tasks.join_next().await.is_some() {}

    let elapsed = start.elapsed();

    // Read final value - it will be one of the writers (last write wins)
    let final_data = storage.get("contended-key").await.unwrap();

    println!("\n=== Write Contention (Same Key) ===");
    println!("Concurrent writers: {}", num_tasks);
    println!("Total time: {:?}", elapsed);
    println!("Final data byte: {} (indeterminate due to race)", final_data[0]);
    println!("⚠️  This test demonstrates race condition - no write coordination");
}

/// Sustained throughput test - measure ops/sec over duration
#[tokio::test]
async fn sustained_throughput() {
    let tmp = tempdir().unwrap();
    let storage = Arc::new(FileStorage::new(tmp.path()).await.unwrap());
    let data = vec![0xAB; 10 * 1024]; // 10KB per object
    let duration = Duration::from_secs(5);

    let start = Instant::now();
    let mut counter = 0u64;
    let mut tasks = JoinSet::new();

    // Spawn tasks continuously for the duration
    let storage_clone = Arc::clone(&storage);
    let producer = tokio::spawn(async move {
        let mut task_id = 0u64;
        loop {
            if start.elapsed() >= duration {
                break;
            }

            let storage = Arc::clone(&storage_clone);
            let data = data.clone();
            let key = format!("sustained-{}", task_id);

            tasks.spawn(async move {
                storage.put(&key, &data).await.unwrap();
            });

            task_id += 1;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        // Wait for remaining tasks
        while tasks.join_next().await.is_some() {
            task_id += 1;
        }
        task_id
    });

    counter = producer.await.unwrap();
    let elapsed = start.elapsed();
    let ops_per_sec = counter as f64 / elapsed.as_secs_f64();

    println!("\n=== Sustained Throughput Test ===");
    println!("Duration: {:?}", elapsed);
    println!("Total operations: {}", counter);
    println!("Throughput: {:.2} ops/sec", ops_per_sec);
    println!("Data written: {:.2} MB", (counter * 10 * 1024) as f64 / (1024.0 * 1024.0));
}

/// Large object handling - test memory usage patterns
#[tokio::test]
async fn large_object_handling() {
    let tmp = tempdir().unwrap();
    let storage = FileStorage::new(tmp.path()).await.unwrap();

    let sizes = vec![
        ("1MB", 1024 * 1024),
        ("10MB", 10 * 1024 * 1024),
        ("50MB", 50 * 1024 * 1024),
    ];

    println!("\n=== Large Object Handling ===");

    for (name, size) in sizes {
        let data = vec![0xAB; size];
        let key = format!("large-{}", name);

        // Measure PUT
        let start = Instant::now();
        storage.put(&key, &data).await.unwrap();
        let put_time = start.elapsed();

        // Measure GET
        let start = Instant::now();
        let retrieved = storage.get(&key).await.unwrap();
        let get_time = start.elapsed();

        assert_eq!(retrieved.len(), size);

        println!("\n{} object:", name);
        println!("  PUT: {:?} ({:.2} MB/s)",
                 put_time,
                 size as f64 / (1024.0 * 1024.0) / put_time.as_secs_f64());
        println!("  GET: {:?} ({:.2} MB/s)",
                 get_time,
                 size as f64 / (1024.0 * 1024.0) / get_time.as_secs_f64());

        // Clean up
        storage.delete(&key).await.unwrap();
    }
}

/// Scalability test - many small objects
#[tokio::test]
async fn many_small_objects() {
    let tmp = tempdir().unwrap();
    let storage = FileStorage::new(tmp.path()).await.unwrap();
    let data = vec![0xAB; 1024]; // 1KB each
    let num_objects = 1000;

    let start = Instant::now();

    for i in 0..num_objects {
        let key = format!("small-{:06}", i);
        storage.put(&key, &data).await.unwrap();
    }

    let write_time = start.elapsed();

    // Random reads
    let start = Instant::now();
    for i in (0..num_objects).step_by(10) {
        let key = format!("small-{:06}", i);
        storage.get(&key).await.unwrap();
    }
    let read_time = start.elapsed();

    println!("\n=== Many Small Objects ===");
    println!("Objects: {}", num_objects);
    println!("Size: 1KB each");
    println!("Write time: {:?} ({:.2} ops/sec)",
             write_time,
             num_objects as f64 / write_time.as_secs_f64());
    println!("Read time (10% sample): {:?}", read_time);
}
