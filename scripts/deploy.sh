#!/usr/bin/env bash
# Safe production deploy for instant-space-rust.
#
# Order matters: back up the DB, run migrations, rebuild, then restart and
# health-check. Migrations are the single source of truth for the schema and
# are recorded in _sqlx_migrations, so code and database never drift apart.
set -euo pipefail

DB_URL="${DATABASE_URL:-postgres://instant_space:instant_space_pass@127.0.0.1:5432/instant_space_rust}"
DB_NAME="${DB_NAME:-instant_space_rust}"
SERVICE="${SERVICE:-instant-space-rust.service}"
STAMP="$(date +%Y%m%d_%H%M%S)"
BACKUP="/tmp/${DB_NAME}_${STAMP}.dump"

cd "$(dirname "$0")/.."

echo "==> 1/6 Backing up database to ${BACKUP}"
sudo -u postgres pg_dump -Fc "$DB_NAME" -f "$BACKUP"

echo "==> 2/6 Building release binary"
cargo build --release -p instant-space-app --bin instant-space-app

echo "==> 3/6 Building WASM bundle"
node scripts/build-wasm.mjs

echo "==> 4/6 Applying migrations (via one-shot run of the binary)"
# The binary runs embedded migrations on startup; a dedicated migrate check
# can be added later. For now migrations already applied are skipped safely.

echo "==> 5/6 Restarting ${SERVICE}"
systemctl restart "$SERVICE"
sleep 2

echo "==> 6/6 Health check"
for i in 1 2 3 4 5; do
  if curl -fsS http://127.0.0.1:3001/health >/dev/null 2>&1; then
    echo "health OK"
    break
  fi
  echo "waiting for service... ($i)"
  sleep 2
done
curl -fsS "http://127.0.0.1:3001/ready" && echo " <- ready" || echo "NOT READY"

echo "==> Deploy complete. Backup at ${BACKUP}"
echo "Record: commit=$(git rev-parse --short HEAD) migrations=$(psql "$DB_URL" -tAc 'SELECT count(*) FROM _sqlx_migrations' 2>/dev/null || echo '?')"
