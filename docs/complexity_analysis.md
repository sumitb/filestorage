# Big O Complexity Analysis

## Overview

This document analyzes the time and space complexity of the FileStorage implementation in `crates/filestorage-core/src/lib.rs`.

## Variables Definition

- `n` = data/file size in bytes
- `k` = key string length (characters)
- `d` = directory depth (number of path segments in key)
- `m` = total number of objects stored in the system

## PUT Operation

**Function**: `FileStorage::put(key: &str, data: &[u8])`

### Call Stack Analysis

```rust
put() → path_for() → validate_key() → create_dir_all() → fs::write()
```

### Line-by-Line Breakdown

**lib.rs:22** - `path_for(key)`
- Calls `validate_key(key)` - **O(k)**

**lib.rs:58-82** - `validate_key(key)`
- Line 59: Empty check - **O(1)**
- Line 63: `Path::new(key)` - **O(1)** (reference, no copy)
- Line 64: `is_absolute()` check - **O(1)**
- Line 70-79: Loop over `path.components()` - **O(d)** where d = number of path segments
  - Each iteration does pattern matching - **O(1)**
- **Total**: **O(d)** (k and d are related: d ≤ k, since d = segments and k = total chars)

**lib.rs:23-24** - `create_dir_all(parent)`
- Creates up to `d` nested directories if they don't exist
- **Time**: **O(d)** - filesystem operations for each directory level
- **Space**: **O(d)** - metadata for directories on disk

**lib.rs:26** - `fs::write(path, data)`
- Writes `n` bytes to disk
- **Time**: **O(n)** - disk I/O proportional to data size
- **Space**: **O(n)** - data stored on disk

### PUT Complexity Summary

| Metric | Complexity | Dominant Term |
|--------|------------|---------------|
| **Time** | **O(k + d + n)** | **O(n)** for large files |
| **Space (Memory)** | **O(1)** | Uses slice reference, no allocation |
| **Space (Disk)** | **O(n + d)** | **O(n)** for data, O(d) for dirs |

**Notes**:
- For typical use: `k < 256`, `d < 10`, but `n` can be GB+
- Time complexity dominated by disk I/O for data writes
- Memory efficient: doesn't copy data

---

## GET Operation

**Function**: `FileStorage::get(key: &str) -> Vec<u8>`

### Call Stack Analysis

```rust
get() → path_for() → validate_key() → fs::read()
```

### Line-by-Line Breakdown

**lib.rs:31** - `path_for(key)`
- Same as PUT: **O(k)**

**lib.rs:32-38** - `fs::read(path)`
- Reads entire file from disk into memory
- **Time**: **O(n)** - disk I/O proportional to file size
- **Space**: **O(n)** - allocates `Vec<u8>` to hold entire file

### GET Complexity Summary

| Metric | Complexity | Dominant Term |
|--------|------------|---------------|
| **Time** | **O(k + n)** | **O(n)** for large files |
| **Space (Memory)** | **O(n)** | ⚠️ **Entire file loaded into RAM** |
| **Space (Disk)** | **O(0)** | Read-only, no disk writes |

**Critical Issue**:
- **Memory bottleneck**: A 1GB file requires 1GB RAM
- **No streaming support**: Cannot handle files larger than available memory
- **Recommendation**: Implement streaming for large files

---

## DELETE Operation

**Function**: `FileStorage::delete(key: &str)`

### Call Stack Analysis

```rust
delete() → path_for() → validate_key() → fs::remove_file()
```

### Line-by-Line Breakdown

**lib.rs:42** - `path_for(key)`
- Same as PUT/GET: **O(k)**

**lib.rs:43-48** - `fs::remove_file(path)`
- Removes file metadata (inode deletion)
- **Time**: **O(1)** - filesystem metadata update (typically)
- **Space**: **O(0)** - frees disk space

**Note**: Actual disk block freeing may be deferred by filesystem, but that's transparent to the operation.

### DELETE Complexity Summary

| Metric | Complexity | Dominant Term |
|--------|------------|---------------|
| **Time** | **O(k)** | Linear in key length |
| **Space (Memory)** | **O(1)** | Minimal stack allocation |
| **Space (Disk)** | **-O(n)** | Frees `n` bytes |

---

## Overall System Complexity

### Storage Capacity

Given `m` objects with average size `n_avg`:

| Metric | Complexity |
|--------|------------|
| **Total Disk Usage** | **O(m × n_avg)** |
| **Metadata Overhead** | **O(m × k_avg)** for directory structure |

### Concurrency Characteristics

**Current Implementation**: No explicit concurrency control

| Scenario | Behavior | Issue |
|----------|----------|-------|
| Concurrent reads (same key) | ✅ Safe | Multiple processes can read simultaneously |
| Concurrent writes (different keys) | ✅ Safe | Independent filesystem operations |
| Concurrent writes (same key) | ⚠️ **Race condition** | Last write wins, potential corruption |
| Read during write (same key) | ⚠️ **Undefined** | May read partial/corrupted data |

---

## Bottlenecks & Optimization Opportunities

### 1. GET Memory Usage - **CRITICAL**

**Problem**: `fs::read()` loads entire file into memory (lib.rs:32)

**Impact**:
- Cannot handle files > available RAM
- Memory pressure for concurrent large file reads
- O(n) memory allocation overhead

**Solutions**:
```rust
// Option A: Streaming API
pub async fn get_stream(&self, key: &str) -> impl Stream<Item = Result<Bytes>>

// Option B: Chunked reads
pub async fn get_chunk(&self, key: &str, offset: u64, size: usize) -> Result<Bytes>
```

**Complexity Improvement**: O(n) → O(chunk_size)

### 2. Concurrent Write Safety

**Problem**: No locking/coordination for same-key writes

**Impact**:
- Data corruption possible
- Non-atomic updates

**Solutions**:
- Add file locking (flock/advisory locks)
- Use atomic rename pattern (write to temp, rename)
- Add distributed locking for multi-node scenarios

### 3. Directory Creation Overhead

**Problem**: `create_dir_all()` on every PUT (lib.rs:24)

**Impact**: Minimal for existing dirs, but adds latency

**Solutions**:
- Cache existence of parent directories
- Lazy directory creation

**Complexity Improvement**: O(d) → O(1) amortized

### 4. Key Validation

**Problem**: Validates key on every operation

**Impact**: Negligible (k typically < 256)

**Status**: Not worth optimizing unless profiling shows it's a hotspot

---

## Comparison: Filesystem vs In-Memory

| Operation | Filesystem (Current) | In-Memory HashMap |
|-----------|---------------------|-------------------|
| **PUT time** | O(n) - disk I/O | O(n) - copy data |
| **GET time** | O(n) - disk I/O | O(1) - hash lookup |
| **DELETE time** | O(k) | O(1) - hash lookup |
| **Memory usage** | O(1) - metadata only | O(Σn) - all data in RAM |
| **Persistence** | ✅ Survives restarts | ❌ Lost on crash |
| **Capacity** | ~TB (disk limited) | ~GB (RAM limited) |

**Tradeoffs**:
- Filesystem: Better for large datasets, persistent, slower
- In-memory: Faster, limited capacity, requires separate persistence

---

## Summary Table

| Operation | Time Complexity | Space (Memory) | Space (Disk) | Main Bottleneck |
|-----------|----------------|----------------|--------------|-----------------|
| **PUT** | O(k + d + n) | O(1) | O(n + d) | Disk I/O |
| **GET** | O(k + n) | **O(n)** ⚠️ | O(0) | **Memory allocation** |
| **DELETE** | O(k) | O(1) | -O(n) | Minimal |

**Key Takeaways**:
1. ✅ PUT is memory-efficient (streams to disk)
2. ⚠️ GET is memory-inefficient (loads entire file)
3. ✅ DELETE is fast and lightweight
4. ⚠️ No concurrency protection for same-key operations
5. All operations scale linearly with data size (optimal for I/O-bound tasks)

---

## Recommended Next Steps

1. **High Priority**: Implement streaming GET to fix O(n) memory issue
2. **Medium Priority**: Add file locking for concurrent write safety
3. **Low Priority**: Profile and optimize if key validation shows up as bottleneck
4. **Future**: Consider sharding/partitioning for very large object counts (m > 1M)
