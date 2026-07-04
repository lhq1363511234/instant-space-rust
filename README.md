# Instant Space Rust

Independent full-stack Rust rewrite of Instant Space.

## Commands

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
sqlx migrate run --source crates/db/migrations
```

## Importer

The importer reads the existing SQLite database and must preserve all source rows:

- spaces: 115
- guides: 210
- users: 1

Representative seed rows are for smoke testing only. Full migration work must keep every source space, guide, and derived location value.
