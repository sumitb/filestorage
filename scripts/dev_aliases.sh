#!/usr/bin/env bash
# Source this file (do not execute) to load handy cargo aliases.
#   source scripts/dev_aliases.sh

alias cchk="cargo check"
alias cfmt="cargo fmt"
alias clint="cargo clippy --all-targets --all-features"
alias ctest="cargo test --all"
alias crun="cargo run -p filestorage --"
# Runs the storage node bound to all interfaces with a temp data dir for quick manual testing.
alias crun_local="FILESTORAGE_ADDR=0.0.0.0:9000 FILESTORAGE_DATA_DIR=/tmp/storage cargo run -p filestorage --"
alias cbuild="cargo build --release"
