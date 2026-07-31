# InSpace Agent API 入门教程（口语版）

> 这篇是给「你」和「你的 AI Agent」看的操作手册，跟着做就能跑通：
> 建一个空间 → 给空间写攻略 → 把攻略发布 → 读回来看效果 → 改内容 → 删掉不要的。
> 技术细节（字段全表、错误码）在 `AGENT_REST_API.md`，这篇只讲怎么用。

---

## 0. 一句话理解

这套 API 让**你自己的程序**（或 AI Agent）代替人，去 InSpace 里开空间、写攻略、管内容。
它跟你平时在网站上点按钮做的事一样，只是换成了 `curl` / HTTP 请求。

- 地址：`https://opctoai.com`
- 路径：`/api/spaces` 管空间，`/api/guides` 管攻略
- 每次请求都要带一个 API Key（钥匙），证明「你是谁」

---

## 1. 第一步：拿一把钥匙（API Key）

钥匙要在**服务器上**生成一次，生成后只显示一遍，请立刻存好。

```bash
cd /root/opt/instant-space-rust

# 取数据库连接（不用改，直接复制）
DBURL=$(systemctl show instant-space-rust -p Environment --value \
  | tr ' ' '\n' | sed -n 's/^DATABASE_URL=//p')

# 给哪个账号发钥匙：把邮箱换成你自己的 InSpace 账号邮箱
DATABASE_URL="$DBURL" target/release/create-agent-key \
  你的邮箱@example.com \
  my-first-agent \
  'spaces:read,spaces:write,guides:read,guides:write'
```

你会看到类似：

```
InSpace Agent API key created for 你的邮箱@example.com (my-first-agent).
Save it now; it will not be shown again:
isp_live_XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

把 `isp_live_...` 那一长串存到你的 Agent 环境变量里：

```bash
export INSPACE_API_KEY='isp_live_这里填你的钥匙'
```

> 钥匙只能看这一次。丢了就再建一把，旧的可以之后在数据库里吊销。
> 钥匙的权限分四种：`spaces:read` 看空间、`spaces:write` 改空间、`guides:read` 看攻略、`guides:write` 改攻略。上面一把钥匙全给了。

---

## 2. 验证钥匙有没有用

```bash
# 应该返回 200 和一堆 JSON（你账号名下的空间列表，可能为空）
curl -H "Authorization: Bearer $INSPACE_API_KEY" \
  'https://opctoai.com/api/spaces?limit=5'
```

返回 `{"items":[...],"limit":5,"offset":0}` 就成功了。
如果返回 `401`，说明钥匙没写对或已失效。

---

## 3. 建一个空间

空间 = 地图上的一个真实地点。建空间必填：名字、省、市、坐标、类型。

```bash
curl -X POST https://opctoai.com/api/spaces \
  -H "Authorization: Bearer $INSPACE_API_KEY" \
  -H 'Content-Type: application/json' \
  -d '{
    "name_zh": "老张瓦罐汤（红谷滩店）",
    "country": "China",
    "province": "江西省",
    "city": "南昌市",
    "spot_name": "红谷滩万达广场店",
    "address_line": "红谷滩区万达广场1楼",
    "lat": 28.68,
    "lng": 115.86,
    "space_type": "food",
    "description_zh": "南昌本地老字号瓦罐汤，招牌是墨鱼汤和排骨汤。",
    "tag_zh": "本地美食,瓦罐汤",
    "is_public": true,
    "duration_hours": 720
  }'
```

- `space_type` 取值：`scenic` 景点 / `food` 美食 / `park` 公园 / `transit` 交通 / `event` 活动 / `custom` 自定义
- `duration_hours`：空间存活时长，720 = 30 天
- 返回里 `password` 是这个空间的门禁密码（只返回一次），`hotspot_name` 是它的 Wi-Fi 热点名，要存好

```json
{
  "id": "5e7e8d24-...",
  "name_zh": "老张瓦罐汤（红谷滩店）",
  "password": "975150",
  "hotspot_name": "InstantSpace_975150"
}
```

**记下返回的 `id`**，后面所有操作都用它。

---

## 4. 给空间写一篇攻略（志）

攻略 = 关于这个地点的内容。可以绑到某个空间上（推荐，读者能互相跳转）。

```bash
curl -X POST https://opctoai.com/api/guides \
  -H "Authorization: Bearer $INSPACE_API_KEY" \
  -H 'Content-Type: application/json' \
  -d '{
    "title_zh": "第一次来南昌，怎么喝瓦罐汤",
    "summary_zh": "给外地朋友的三条点单建议",
    "content_zh": "早上先喝墨鱼汤，中午加一份拌粉……",
    "province": "江西省",
    "city": "南昌市",
    "spot_name": "红谷滩万达广场店",
    "space_id": "5e7e8d24-这里填空间的id",
    "sections": [
      {
        "type": "text",
        "title_zh": "点单",
        "content_zh": "墨鱼汤配拌粉是经典组合",
        "images": []
      }
    ],
    "status": "draft"
  }'
```

要点：

- **`status`**：`draft`（草稿，别人看不到）/ `published`（发布）/ `archived`（下线）。想直接上线就写 `published`。
- **`sections` 的字段名必须长这样**：`type`、`title_zh`、`content_zh`、`images`。
  ⚠️ 别写成 `heading_zh` / `body_zh`，那样会被悄悄丢掉。
- `space_id` 不写也行，攻略就是「独立攻略」；写了就和空间绑定。

---

## 5. 读回来看内容

列表接口只给「摘要卡片」（标题、地点、状态），要看全文用详情接口。

```bash
# 攻略详情（有 sections / content / images）
curl -H "Authorization: Bearer $INSPACE_API_KEY" \
  'https://opctoai.com/api/guides/<攻略id>'

# 空间详情（有描述 / 标签 / 主理人）
curl -H "Authorization: Bearer $INSPACE_API_KEY" \
  'https://opctoai.com/api/spaces/<空间id>'
```

### 搜索

```bash
# 多关键词 AND 搜索（两个词都要命中）
curl -H "Authorization: Bearer $INSPACE_API_KEY" \
  'https://opctoai.com/api/guides?q=南昌%20瓦罐汤&limit=20'

# 分页：limit 最多 100，offset 从 0 开始
curl -H "Authorization: Bearer $INSPACE_API_KEY" \
  'https://opctoai.com/api/spaces?limit=50&offset=50'
```

注意：列表接口只能看到**你（钥匙所属账号）名下**的空间和攻略。

---

## 6. 改内容 / 发布

PATCH 是「只改你传的字段」，没传的保持不变。

```bash
# 把草稿发布
curl -X PATCH https://opctoai.com/api/guides/<攻略id> \
  -H "Authorization: Bearer $INSPACE_API_KEY" \
  -H 'Content-Type: application/json' \
  -d '{"status": "published"}'

# 改攻略标题
curl -X PATCH https://opctoai.com/api/guides/<攻略id> \
  -H "Authorization: Bearer $INSPACE_API_KEY" \
  -H 'Content-Type: application/json' \
  -d '{"title_zh": "南昌瓦罐汤点单指南（更新版）"}'

# 改空间描述
curl -X PATCH https://opctoai.com/api/spaces/<空间id> \
  -H "Authorization: Bearer $INSPACE_API_KEY" \
  -H 'Content-Type: application/json' \
  -d '{"description_zh": "新描述", "tag_zh": "本地美食"}'
```

---

## 7. 删除

```bash
# 删攻略（永久）
curl -X DELETE -H "Authorization: Bearer $INSPACE_API_KEY" \
  'https://opctoai.com/api/guides/<攻略id>'

# 删空间（永久，会连带删掉它下面的攻略/聊天/故事）
curl -X DELETE -H "Authorization: Bearer $INSPACE_API_KEY" \
  'https://opctoai.com/api/spaces/<空间id>'
```

成功返回 `204`（没有内容）。删除不可恢复；只想「下架不删」就把攻略 `status` 改成 `archived`。

---

## 8. 给 AI Agent 用的 Python 示例

```python
import os, requests

BASE = "https://opctoai.com"
HEADERS = {"Authorization": f"Bearer {os.environ['INSPACE_API_KEY']}"}

# 1. 建空间
space = requests.post(f"{BASE}/api/spaces", headers=HEADERS, json={
    "name_zh": "AI 演示空间",
    "province": "江西省", "city": "南昌市",
    "lat": 28.68, "lng": 115.86,
    "space_type": "food", "is_public": True, "duration_hours": 720,
}).json()
space_id = space["id"]
print("空间已建:", space_id, "密码:", space["password"])

# 2. 写攻略并发布
guide = requests.post(f"{BASE}/api/guides", headers=HEADERS, json={
    "title_zh": "AI 生成的攻略",
    "province": "江西省", "city": "南昌市",
    "space_id": space_id,
    "sections": [{"type": "text", "title_zh": "简介", "content_zh": "由 Agent 自动生成"}],
    "status": "published",
}).json()
guide_id = guide["id"]

# 3. 读回全文
detail = requests.get(f"{BASE}/api/guides/{guide_id}", headers=HEADERS).json()
print("攻略内容:", detail["content_zh"])

# 4. 删掉（演示完清理）
requests.delete(f"{BASE}/api/guides/{guide_id}", headers=HEADERS)
requests.delete(f"{BASE}/api/spaces/{space_id}", headers=HEADERS)
```

---

## 9. 常见报错速查

| 状态码 | 意思 | 怎么办 |
|---|---|---|
| 401 | 钥匙不对 | 检查 `$INSPACE_API_KEY`，或重新建一把 |
| 403 | 权限不够 | 这把钥匙没有对应 scope；或想改/删的不是你名下的对象 |
| 404 | 对象不存在 | id 写错了，或已被删除 |
| 429 | 限流了 | 默认每分钟 60 次，等一下再发 |
| 400 | 参数错误 | 看返回 message；必填字段没填或坐标越界 |

所有报错返回格式都是：

```json
{"error": {"code": "not_found", "message": "guide not found"}}
```

---

## 10. 一个完整的运营流程（照着写就行）

1. 建 Key（第 1 节）→ 2. 建空间（第 3 节，存好 id 和密码）
3. 给空间写 1 篇攻略，`status: published`（第 4 节）
4. 读回详情检查排版（第 5 节）
5. 每天定时：搜索已发布攻略 → 发现过期的就 PATCH 更新（第 6 节）
6. 主理人不要的空间/攻略 → DELETE 清理（第 7 节）
