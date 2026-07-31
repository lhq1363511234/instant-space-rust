# 备份与恢复演练（inspace / SpaceOS）

> 目的：生产数据不丢，恢复步骤可演练。本文是操作手册，不是宣传文档。
> 适用对象：服务器运维（root / systemd 服务 `instant-space-rust`）。

## 1. 数据都在哪

| 数据 | 位置 | 说明 |
|---|---|---|
| 业务数据（用户/空间/攻略/聊天/胶囊…） | PostgreSQL 数据库 | 唯一权威数据源 |
| 首页 CMS 版本 | 同库 `site_page_configs` / `site_page_versions` | 库内 |
| 攻略图片（Phase 4 上传） | `/root/opt/instant-space-rust/uploads/` | 磁盘文件，需随库一起备份 |
| 代码与迁移 | git + `crates/db/migrations/` | 部署靠迁移升级，不靠手工 DDL |
| 系统配置 | `/etc/systemd/system/instant-space-rust.service`（或 unit 文件）、nginx | 少量，随主机快照即可 |

## 2. 每日备份（推荐 cron）

```bash
#!/usr/bin/env bash
set -euo pipefail
STAMP="$(date +%Y%m%d-%H%M%S)"
DIR=/var/backups/inspace
mkdir -p "$DIR"
# 全库逻辑备份（含 schema）
pg_dump "postgres://.../inspace" -Fc -f "$DIR/inspace-$STAMP.dump"
# 上传媒体目录（rsync 到异地/对象存储）
tar -C /root/opt/instant-space-rust -czf "$DIR/inspace-uploads-$STAMP.tgz" uploads
# 保留最近 14 天，删除更早
find "$DIR" -name 'inspace-*.dump' -mtime +14 -delete
find "$DIR" -name 'inspace-uploads-*.tgz' -mtime +14 -delete
```

cron 示例（每天 03:17 UTC）：

```cron
17 3 * * * /usr/local/bin/inspace-backup.sh >> /var/log/inspace-backup.log 2>&1
```

## 3. 恢复演练（每季度至少一次，在测试库上做）

```bash
# 1) 建空库
createdb inspace_restore_test
# 2) 灌入备份
pg_restore -d inspace_restore_test --no-owner --role=inspace_app inspace-20260731-030000.dump
# 3) 校验行数与 schema contract
psql inspace_restore_test -c "SELECT count(*) FROM spaces;"
psql inspace_restore_test -c "SELECT count(*) FROM guides;"
# 4) 用恢复库启动一次服务（仅本地端口），确认 /ready 200 且日志无 schema drift
DATABASE_URL=postgres://.../inspace_restore_test /usr/local/bin/instant-space-app
curl -s http://127.0.0.1:3001/ready
# 5) 上传目录按同一时间点恢复，抽查一张攻略图 URL 能打开
```

恢复要点：

- `pg_restore` 用 `--no-owner`，应用角色权限与迁移在部署时重建，不要在 dump 里硬编码角色。
- 上传目录的时间点要和库的 dump 时间点尽量一致，否则会出现图片 URL 存在但文件缺失。
- 服务启动会自动跑迁移 + schema contract 校验；恢复后看到 `schema contract verified` 才算干净。

## 4. 迁移漂移自检（每次部署）

服务启动日志会自动打印：

- `migrations applied` → 迁移正常执行
- `schema contract verified` → 表/列契约齐全
- `schema drift detected — missing items: ...` → **立即停机排查**，按 `docs/SITE_MODULE_ARCHITECTURE.md` 第 2.3 节流程修，不要带着漂移继续跑。

## 5. 灾难恢复顺序

1. 停服务（避免写库与旧 schema 打架）。
2. 恢复数据库（步骤 3.2）。
3. 恢复 `uploads/` 目录。
4. 启动服务，看日志：`migrations applied` + `schema contract verified`。
5. `curl /health` 与 `/ready`；再抽查首页与一个空间详情页。
6. 恢复完成后把本次演练记录追加到 `memory.md`。
