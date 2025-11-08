use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use filestorage_core::FileStorage;
use std::time::Duration;
use tempfile::tempdir;

// Helper to generate test data of specific size
fn generate_data(size: usize) -> Vec<u8> {
    vec![0xAB; size]
}

// Benchmark PUT operations with varying data sizes
fn bench_put(c: &mut Criterion) {
    let mut group = c.benchmark_group("put");

    let sizes = vec![
        ("1KB", 1024),
        ("10KB", 10 * 1024),
        ("100KB", 100 * 1024),
        ("1MB", 1024 * 1024),
        ("10MB", 10 * 1024 * 1024),
    ];

    for (name, size) in sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, &size| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let tmp = tempdir().unwrap();
            let storage = runtime.block_on(FileStorage::new(tmp.path())).unwrap();
            let data = generate_data(size);

            b.to_async(&runtime).iter(|| async {
                storage
                    .put(black_box("test-object"), black_box(&data))
                    .await
                    .unwrap()
            });
        });
    }

    group.finish();
}

// Benchmark PUT with nested directory structures
fn bench_put_nested_keys(c: &mut Criterion) {
    let mut group = c.benchmark_group("put_nested");

    let keys = vec![
        ("flat", "object.bin"),
        ("1-level", "dir1/object.bin"),
        ("3-levels", "dir1/dir2/dir3/object.bin"),
        ("5-levels", "a/b/c/d/e/object.bin"),
        ("10-levels", "a/b/c/d/e/f/g/h/i/j/object.bin"),
    ];

    let data = generate_data(1024); // 1KB data

    for (name, key) in keys {
        group.bench_with_input(BenchmarkId::from_parameter(name), &key, |b, &key| {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            b.to_async(&runtime).iter(|| async {
                // Create new storage for each iteration to measure dir creation
                let tmp = tempdir().unwrap();
                let storage = FileStorage::new(tmp.path()).await.unwrap();
                storage.put(black_box(key), black_box(&data)).await.unwrap()
            });
        });
    }

    group.finish();
}

// Benchmark GET operations with varying data sizes
fn bench_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("get");

    let sizes = vec![
        ("1KB", 1024),
        ("10KB", 10 * 1024),
        ("100KB", 100 * 1024),
        ("1MB", 1024 * 1024),
        ("10MB", 10 * 1024 * 1024),
    ];

    for (name, size) in sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, &size| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let tmp = tempdir().unwrap();
            let storage = runtime.block_on(FileStorage::new(tmp.path())).unwrap();
            let data = generate_data(size);

            // Pre-populate storage
            runtime.block_on(storage.put("test-object", &data)).unwrap();

            b.to_async(&runtime).iter(|| async {
                black_box(storage.get(black_box("test-object")).await.unwrap())
            });
        });
    }

    group.finish();
}

// Benchmark DELETE operations
fn bench_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("delete");

    let data = generate_data(1024); // 1KB data

    group.bench_function("delete", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();

        b.to_async(&runtime).iter(|| async {
            let tmp = tempdir().unwrap();
            let storage = FileStorage::new(tmp.path()).await.unwrap();
            storage.put("test-object", &data).await.unwrap();
            storage.delete(black_box("test-object")).await.unwrap()
        });
    });

    group.finish();
}

// Benchmark key validation overhead
fn bench_key_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_validation");

    let keys = vec![
        ("short", "a"),
        ("medium", "path/to/some/object.bin"),
        ("long", "very/deep/nested/directory/structure/with/many/segments/object.bin"),
        ("very-long", "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z/object.bin"),
    ];

    let data = generate_data(1024);

    for (name, key) in keys {
        group.bench_with_input(BenchmarkId::from_parameter(name), &key, |b, &key| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let tmp = tempdir().unwrap();
            let storage = runtime.block_on(FileStorage::new(tmp.path())).unwrap();

            b.to_async(&runtime).iter(|| async {
                storage.put(black_box(key), black_box(&data)).await.unwrap()
            });
        });
    }

    group.finish();
}

// Benchmark round-trip operations (PUT -> GET -> DELETE)
fn bench_round_trip(c: &mut Criterion) {
    let mut group = c.benchmark_group("round_trip");

    let sizes = vec![
        ("1KB", 1024),
        ("100KB", 100 * 1024),
        ("1MB", 1024 * 1024),
    ];

    for (name, size) in sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(name), &size, |b, &size| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let data = generate_data(size);

            b.to_async(&runtime).iter(|| async {
                let tmp = tempdir().unwrap();
                let storage = FileStorage::new(tmp.path()).await.unwrap();

                storage.put(black_box("object"), black_box(&data)).await.unwrap();
                let retrieved = storage.get(black_box("object")).await.unwrap();
                black_box(retrieved);
                storage.delete(black_box("object")).await.unwrap();
            });
        });
    }

    group.finish();
}

// Configure criterion
criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .sample_size(50);
    targets = bench_put, bench_put_nested_keys, bench_get, bench_delete,
              bench_key_validation, bench_round_trip
}

criterion_main!(benches);
