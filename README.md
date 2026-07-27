# Instant Space Rust

Independent full-stack Rust rewrite of Instant Space.

## Product / Architecture

Product baseline: [`docs/PRODUCT_VISION.md`](docs/PRODUCT_VISION.md). Current focus is **Global Travel Guide Spaces**: the map shows Spaces, a Space can contain many Guides, and Guides may also exist as standalone content.

Rust-specific architecture and roadmap: [`docs/ARCHITECTURE_RUST.md`](docs/ARCHITECTURE_RUST.md). This document uses the legacy Node/Next.js project only as product context and describes the independent Rust implementation.

Phase plan: [`docs/PHASES.md`](docs/PHASES.md).

Maintenance module map and current Phase gap audit: [`docs/SITE_MODULE_ARCHITECTURE.md`](docs/SITE_MODULE_ARCHITECTURE.md).

## Commands

```powershell
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
sqlx migrate run --source crates/db/migrations
npm run build:wasm
npm run test:wasm
npm run test:browser
```

## Importer

The importer reads the existing SQLite database and must preserve all source rows:

- spaces: 115
- guides: 210
- users: 1

Representative seed rows are for smoke testing only. Full migration work must keep every source space, guide, and derived location value.

## Handoff

Local app URL: http://127.0.0.1:3001

This repository is an independent Rust rewrite skeleton covering:

- Axum + Leptos workspace structure
- PostgreSQL schema, seeds, and sqlx repository layer
- Leptos SSR shell with WASM hydration build wiring
- MapLibre-oriented homepage and browser smoke tests
- Private space password boundary and chat access policy
- Guide hierarchy, admin statistics, templates, residents, and location boundaries
- SQLite source coverage check for the existing Next.js database

Browser routes now render through the Leptos SSR app instead of static smoke-test HTML. `npm run build:wasm` builds the hydration bundle into `target/site/pkg`, and Axum serves it from `/pkg`.

Final verification run on 2026-07-04:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `DATABASE_URL=postgres://postgres:postgres@localhost:5432/instant_space_rust cargo test --workspace`
- `DATABASE_URL=postgres://postgres:postgres@localhost:5432/instant_space_rust sqlx migrate run --source crates/db/migrations`
- `cargo run -p instant-importer -- ..\china-interactive-map\prisma\dev.db`
- `npm run test:browser`

SSR verification added on 2026-07-04:

- `DATABASE_URL=postgres://postgres:postgres@localhost:5432/instant_space_rust cargo test -p instant-space-app`
- `npm run test:wasm`
- `npm run test:browser`

## License / 许可证

本项目采用 [`PolyForm Noncommercial License 1.0.0`](LICENSE)。允许个人学习、研究、测试及其他许可证定义的非商业用途；**未经许可方事先书面授权，禁止任何商业使用、商业部署、收费集成、销售或商业分发。** 商业授权请联系本仓库所有者。具体权利和限制以 [`LICENSE`](LICENSE) 正文为准。

This project is licensed under the [`PolyForm Noncommercial License 1.0.0`](LICENSE). Commercial use requires prior written authorization from the licensor. See [`LICENSE`](LICENSE) for the controlling terms.
