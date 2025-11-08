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
- `cargo run -p filestorage -- <args>` — executes the CLI; the `crun` alias accepts arguments directly.
