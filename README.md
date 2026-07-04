# Instant Space Rust

Independent full-stack Rust rewrite of Instant Space.

## Commands

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
sqlx migrate run --source crates/db/migrations
```
