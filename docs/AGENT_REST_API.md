# InSpace Agent REST API（作者自用）

> 状态：2026-07-30 已上线。当前只提供 API + API Key 鉴权，不接大模型，不向普通用户开放。

## 目的

让作者自己的 AI Agent 程序化创建、读取和管理空间与「志」（代码/英文协议仍使用 `guide`）。Agent 创建的数据归属于 API Key 绑定的 InSpace 用户。

## 地址与鉴权

公网根地址：`https://opctoai.com`

请求头二选一：

```http
Authorization: Bearer <API_KEY>
```

```http
X-Inspace-Api-Key: <API_KEY>
```

Key 只在创建时显示一次；数据库仅保存 Argon2 哈希和非秘密前缀。

## Scope

- `spaces:read`
- `spaces:write`
- `guides:read`
- `guides:write`
- `*`（仅在确有需要时使用）

初始限流：每个 Key 每分钟 60 次，可在数据库中单独调整；每次通过鉴权的请求都会写入 `agent_api_audit_log`。

## 创建 Key

在服务器执行（不要把输出提交到 Git、日志或聊天记录）：

```bash
cd /root/opt/instant-space-rust
DBURL=$(systemctl show instant-space-rust -p Environment --value \
  | tr ' ' '\n' | sed -n 's/^DATABASE_URL=//p')
DATABASE_URL="$DBURL" target/release/create-agent-key \
  <绑定用户邮箱> \
  author-agent \
  'spaces:read,spaces:write,guides:read,guides:write'
```

## 端点

| 方法 | 路径 | Scope | 作用 |
|---|---|---|---|
| GET | `/api/spaces?q=&limit=&offset=` | `spaces:read` | 读取 Key 用户管理的空间 |
| POST | `/api/spaces` | `spaces:write` | 创建空间；只在响应中返回一次 6 位密码 |
| PATCH | `/api/spaces/:id` | `spaces:write` | 更新 Key 用户管理的空间 |
| GET | `/api/guides?q=&limit=&offset=` | `guides:read` | 读取 Key 用户拥有/管理的志；`q` 支持多关键词 AND |
| POST | `/api/guides` | `guides:write` | 创建志，可绑定 `space_id` |
| PATCH | `/api/guides/:id` | `guides:write` | 更新 Key 用户拥有/管理的志 |

列表单次默认 50 条，最大 100 条。空间/志的 PATCH 只能操作 Key 绑定用户拥有或管理的对象；跨用户对象返回 404/403。

## 最小示例

```bash
curl -H "Authorization: Bearer $INSPACE_API_KEY" \
  'https://opctoai.com/api/guides?q=南昌%20滕王阁&limit=20'
```

```bash
curl -X POST \
  -H "Authorization: Bearer $INSPACE_API_KEY" \
  -H 'Content-Type: application/json' \
  https://opctoai.com/api/spaces \
  -d '{
    "name_zh":"我的餐馆",
    "country":"China",
    "province":"江西省",
    "city":"南昌市",
    "lat":28.68,
    "lng":115.86,
    "space_type":"food",
    "is_public":true,
    "duration_hours":720
  }'
```

## 错误格式

```json
{
  "error": {
    "code": "missing_api_key",
    "message": "API key required"
  }
}
```

常见状态：`400` 参数错误、`401` Key 缺失/无效、`403` scope 或归属不足、`404` 对象不存在、`429` 限流、`500` 服务端错误。
