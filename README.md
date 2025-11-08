# filestorage

Learning-oriented file storage playground built with Rust 2024 edition.

## Workspace Layout

Inspired by [Tokio's](https://github.com/tokio-rs/tokio) multi-crate workspace:

- `crates/filestorage-core/` — reusable library code.
- `crates/filestorage/` — CLI binary that wires the core crate into an executable.
- `examples/` — standalone crates (e.g. `examples/basic`) that demonstrate API usage.
- `tests/` — smoke/integration crates that exercise public APIs end-to-end.
- `docs/` — long-form documentation such as design notes.
- `scripts/` — helper shell aliases (`source scripts/dev_aliases.sh`) for common cargo workflows.

## Commands

- `cargo check` — fast syntax/type validation.
- `cargo fmt && cargo clippy --all-targets --all-features` — formatting and linting across the workspace.
- `cargo test --all` — runs unit, example, and smoke tests.
- `cargo run -p filestorage` — starts the HTTP storage node (or use the `crun` alias).

## Running the storage node

```bash
cargo run -p filestorage
# environment overrides
# FILESTORAGE_ADDR=0.0.0.0:9000 FILESTORAGE_DATA_DIR=/tmp/storage cargo run -p filestorage
# or source scripts/dev_aliases.sh and run `crun_local` for the same configuration
```

Environment variables:

- `FILESTORAGE_ADDR` — socket address to bind (default `127.0.0.1:8080`).
- `FILESTORAGE_DATA_DIR` — filesystem directory for stored objects (default `./data`).

### HTTP API

All endpoints live under `/objects/{key}`:

- `PUT /objects/{key}` — store raw request body under `key`.
- `GET /objects/{key}` — stream back the stored bytes.
- `DELETE /objects/{key}` — remove the object.

Example interaction:

```bash
curl -X PUT localhost:8080/objects/hello -d 'hi axum'
curl localhost:8080/objects/hello            # -> hi axum
curl -X DELETE localhost:8080/objects/hello
```
