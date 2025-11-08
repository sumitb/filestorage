# Repository Guidelines

## Project Structure & Module Organization
This is a standard Cargo workspace: `Cargo.toml` defines package metadata, and `src/main.rs` hosts the binary entrypoint. Create additional modules under `src/` (for example `src/storage.rs`) and expose them through `mod` declarations in `main.rs` or a new `lib.rs`. Keep shared fixtures or schemas in `src/shared/` so both the binary and future integration tests can import them. Integration tests belong in `tests/`, each file acting as its own crate; co-locate sample assets under `tests/data/` to keep fixtures versioned.

## Build, Test, and Development Commands
- `cargo check`: fast type-check to validate new code before committing.
- `cargo fmt && cargo clippy --all-targets --all-features`: formats and lints the project; run before submitting a PR.
- `cargo run -- <args>`: executes the filestorage binary locally.
- `cargo test --all`: runs unit and integration tests; add `-- --nocapture` while debugging to stream logs.

## Coding Style & Naming Conventions
Use rustfmt’s default 4-space indentation and keep lines under 100 characters. Modules and files follow `snake_case` (`file_store.rs`), types use `PascalCase`, and functions/variables use `snake_case`. Prefer explicit return types and `?` over `unwrap()` for recoverable errors. Clippy warnings should be fixed or justified with `#[allow(...)]` plus a comment.

## Testing Guidelines
Place focused unit tests inside the same file as the code under `#[cfg(test)] mod tests`. Favor descriptive names such as `stores_file_to_disk`. Integration tests in `tests/` should exercise public APIs end-to-end and can spin up temporary directories via `tempfile`. Aim to cover success and failure paths for filesystem interactions, especially permission errors and missing directories. Use `cargo test -- --ignored` to run any heavier tests you mark with `#[ignore]`.

## Commit & Pull Request Guidelines
There is no prior history, so adopt Conventional Commits (`feat: add uploader`, `fix: handle missing dirs`) with an imperative subject ≤72 characters and optional body detailing rationale. Each PR should include: a concise summary of the change, linked issue or task ID, reproduction or verification steps, and screenshots/terminal output when UI or CLI behavior changes. Rebase on `master` before requesting review, ensure all `cargo` commands above pass, and request at least one approval before merging.
