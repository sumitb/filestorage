# Architecture

This workspace follows a Tokio-inspired layout:

- **Core crate** (`crates/filestorage-core`) contains reusable logic.
- **Binary crate** (`crates/filestorage`) is a thin CLI entrypoint.
- **Examples** (`examples/`) are standalone crates showcasing API usage.
- **Tests** (`tests/`) bundle integration-style or smoke tests that exercise the public APIs.

Additional crates can be added under `crates/` as the project grows.
