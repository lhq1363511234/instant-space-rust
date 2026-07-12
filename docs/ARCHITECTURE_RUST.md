# Instant Space Rust 架构文档

> 状态：Rust 重写版架构基线  
> 参考来源：旧版 Node/Next.js 项目的产品意图、数据边界和业务流程  
> 原则：不复制旧实现；以 Rust、Leptos、Axum、PostgreSQL 的方式重新设计和演进。

---

## 1. 产品定位

Instant Space 当前聚焦为 **全球旅行攻略与共享体验空间平台**。用户围绕真实旅行地点创建、发现和进入“空间”；空间承载攻略、社群、私密访问、聊天以及未来的二维码/3D 共享体验。

一句话：**每一个真实旅行地点，都可以有一个可通过地图、链接或二维码进入的数字攻略空间。**

产品落地基线见：[`docs/PRODUCT_VISION.md`](./PRODUCT_VISION.md)。架构实现必须服从该文档中的核心关系：

```text
地图默认显示 Space
Space 1 : N Guide
Guide.space_id 可为空，表示独立攻略
Space 详情和管理入口都必须能看到相关 Guide
```

Rust 版的目标不是逐字迁移旧系统，而是保留核心业务模型，并建立更稳定、可测试、可持续扩展的全栈 Rust 架构。

### 1.1 核心对象

- **Space / 空间**：地图上的主对象，代表真实旅行地点的数字入口，包含名称、分类、坐标、公开性、密码版本、生命周期状态和主理人。
- **Host / 主理人**：创建并管理空间的登录用户。
- **Private Space / 私密空间**：需要密码验证后才能进入或聊天；密码版本变化后旧访问失效。
- **Guide / 攻略**：空间下的结构化内容资产；一份攻略可以绑定一个空间，也可以作为 `space_id = NULL` 的独立攻略。
- **Community / 社群**：当前用 Discord/QQ 承接空间讨论组和密码分发，后续可迁移到站内实时互动。
- **QR / Link 入口**：空间的分享和线下扫码入口，用户扫码后应进入 Space 详情页。
- **Admin / 管理员**：负责全局数据管理、空间与攻略运营后台、驻留申请处理；不定位为逐条人工审核员，后期可接入 AI 持续辅助管理。
- **Access Session / 访问凭证**：私密空间验证后的短期权限。
- **Chat / Help / Game**：空间内的实时互动模块，按阶段逐步实现。

---

## 2. 当前 Rust 技术栈

| 层 | 选择 | 说明 |
|---|---|---|
| Web 服务 | Axum | 负责 HTTP 服务、静态资源、Leptos SSR 路由 |
| 前端 | Leptos SSR + WASM Hydration | Rust 组件化 UI，服务端首屏 + 浏览器交互 |
| 数据库 | PostgreSQL | 生产数据库，使用迁移文件管理结构 |
| 数据访问 | sqlx | 编译期友好的异步 SQL 层 |
| 领域模型 | `crates/domain` | 枚举、结构体、生命周期规则、纯业务测试 |
| 认证 | Argon2 + Session Cookie | 密码哈希与服务端会话 |
| 地图 | MapLibre GL JS | 通过 Rust WASM shim 调用浏览器地图库 |
| 地图资源 | 本地 MapLibre + `/inspace/ofm` 代理 | 避免 CDN 和跨域瓦片不稳定 |
| 部署 | systemd + Nginx | App 监听 `127.0.0.1:3001`，Nginx 绑定 `/inspace` |

---

## 3. Workspace 分层

```text
instant-space-rust/
├── app/                     # Axum + Leptos 应用
│   ├── src/app.rs           # Leptos 根组件、路由、SSR shell
│   ├── src/main.rs          # Axum server、静态资源、Leptos routes
│   ├── src/components/      # UI 组件：地图、导航、登录、空间表单等
│   ├── src/pages/           # 页面：Home、Login、My Spaces、Guides、Admin
│   └── src/server/          # Server Functions：auth、spaces、guides、chat、admin
│
├── crates/domain/           # 纯领域模型和业务规则
├── crates/db/               # PostgreSQL migrations 与 repository 查询
├── crates/auth/             # 密码哈希、token 生成等认证基础能力
├── crates/map-ui/           # Rust <-> MapLibre JS 绑定
└── crates/importer/         # 旧数据导入/迁移校验工具
```

### 3.1 分层约束

- `domain` 不依赖数据库和 Web 框架，只描述业务概念和规则。
- `db` 只负责 SQL、迁移和数据转换，不放页面逻辑。
- `app/src/server` 负责把 Leptos Server Function 请求转成数据库调用和权限检查。
- `components/pages` 只处理 UI 状态、交互和资源读取。
- `map-ui` 是浏览器地图适配层，所有 MapLibre 直接调用集中在这里，避免散落在 UI 中。

---

## 4. 路由与页面结构

Rust 版以 `/inspace` 为线上主路径，同时保留本地开发根路径。

| 路由 | 作用 | 状态 |
|---|---|---|
| `/inspace` | 首页地图、空间探索、创建空间弹窗入口 | 已实现 |
| `/inspace/login` | 统一登录 / 注册 | 已实现 |
| `/inspace/my-spaces` | 登录用户的空间管理页 | 基础版已实现 |
| `/inspace/guides` | 攻略浏览与层级筛选 | 基础版已实现 |
| `/inspace/admin` | 管理后台入口 | 基础版已实现 |
| `/admin` | 重定向到 `/inspace/admin` | Nginx 层处理 |

### 4.1 首页原则

首页是地图优先的探索界面，不承载管理后台。登录用户可以看到头像和创建入口，但后台能力必须进入明确的页面或弹窗，避免把 admin 暴露在首页主导航。

### 4.2 创建空间交互

创建空间不是独立页面，而是弹窗：

1. 用户点击顶部“创建空间”。
2. 如果未登录，弹窗提示先登录。
3. 如果已登录，显示空间表单和地图选点器。
4. 用户点击小地图，自动写入经纬度。
5. 提交后空间归属当前用户，并可在“我的空间”中看到。

---

## 5. 数据模型基线

Rust 版使用 PostgreSQL，不复用旧 SQLite/Prisma schema。旧 schema 只用于理解业务字段。

### 5.1 用户与会话

- `users`
  - `email`
  - `name`
  - `password_hash`
  - `role`: `user | admin | super_admin`
- `sessions`
  - 绑定用户或管理员标识
  - 保存 token hash
  - 有过期时间

设计原则：统一用户登录和管理员权限模型；管理员不是首页功能，而是一种角色权限。

### 5.2 空间

- `spaces`
  - 中英文名称
  - 类型：`scenic | food | park | transit | event | custom`
  - 省 / 市 / 区
  - 经纬度
  - 公开性
  - 密码 hash 与密码版本
  - 生命周期：`active | expired | closed | archived | template`
  - 主理人 `host_user_id`
  - 驻留申请字段
  - 社群链接字段

空间状态机：

```text
[active] ─── 时间到期 ───→ [expired]
   ↓                        ↓
   ├── 主理人关闭 ──→ [closed] ──→ [archived] ──→ [template]
   │                        ↑
   └────────────────────────┘
                     主理人重新激活
```

驻留空间流程：

1. 主理人点击“申请驻留”。
2. 管理员收到待审批记录。
3. 管理员与主理人在 Discord/QQ 讨论组内沟通。
4. 管理员审批并设置 `resident_days`。
5. 系统设置 `resident = true`，`expires_at = now() + resident_days`。
6. 驻留空间在地图上优先展示，排序权重建议 `+10`。
7. 到期前 3 天发送 Discord/QQ 提醒。

### 5.3 私密访问

- `access_sessions`
  - 关联空间
  - 记录密码版本
  - 过期后失效

规则：

- 空间密码更新后，旧的访问会话不会立刻把已经在空间页的人踢出。
- 读取空间页和历史消息只要求 access session 存在且未过期。
- 每次发送消息必须校验 access session 记录的 `password_version` 是否等于当前空间密码版本。
- 如果版本不一致，提示重新输入密码；输入错误则不能继续聊天。

### 5.3.1 Discord/QQ 社群密码分发

固定社群结构：

```text
Discord/QQ 频道：即时空间
├── #全局公告
├── #空间列表
├── 【省份板块】广东省
│   ├── 【讨论组】汕头中山公园
│   ├── 【讨论组】广州白云山
│   └── 【讨论组】深圳莲花山
├── 【省份板块】北京市
│   ├── 【讨论组】颐和园
│   └── 【讨论组】故宫
└── ...
```

权限逻辑：

- 所有人可见全局公告和空间列表。
- 用户按省份进入对应板块。
- 用户申请加入具体空间讨论组。
- 主理人审核通过后，成员才能看到实时密码和更新。
- 远程用户可以看公告，但拿不到具体空间的实时密码。

固定社区链接：

- Discord：`https://discord.gg/zsmYWvXyy`
- QQ 频道：`https://pd.qq.com/s/8ru51ih0m?b=9`

每个空间详情页底部展示社区入口，并提示用户在社群内搜索空间名获取进入密码。

### 5.4 攻略

- `guides`
  - 中英文标题、摘要、正文
  - `province / city / district / spot_name`
  - `images` 和 `sections` 使用 `jsonb`
  - 状态：`draft | published | archived`
  - `space_id` 可为空；为空表示独立攻略，非空表示绑定到一个空间
  - `author_id` 用于判断普通用户是否可编辑/删除

攻略层级数据来自数据库字段聚合，而不是硬编码。

空间与攻略关系：

```text
Space 1 : N Guide
Guide.space_id optional
```

产品规则：

- 地图默认显示 Space，而不是 Guide。
- Space 详情页展示绑定到该空间的攻略。
- 管理 Space 时也必须能管理相关攻略。
- 写攻略时必须能选择“绑定已有空间”或“独立攻略”。
- 从某个 Space 入口创建攻略时，应自动预选该 `space_id`。
- 已存在攻略再次保存必须更新同一条记录，不能重复创建多份。

### 5.5 互动模块

- `chat_messages`：空间聊天消息，当前已有基础验证边界。
- `helps`：空间内求助。
- `games`：空间内游戏状态。

这些表是后续实时功能的基础，不应在前端模拟长期保存。

---

## 6. Server Functions 与 API 边界

当前 Leptos Server Functions 挂载在 `/inspace/api` 下。前端组件通过类型化函数调用服务端，而不是手写 REST 请求。

### 6.1 Auth

| Function | 作用 |
|---|---|
| `register_user` | 注册普通用户并创建会话 |
| `login_user` | 登录并创建会话 |
| `current_session` | 获取当前登录用户 |
| `require_admin_user` | SSR 内部管理员权限检查 |

### 6.2 Spaces

| Function | 作用 |
|---|---|
| `list_spaces` | 首页地图空间列表，支持搜索和类型筛选 |
| `create_space` | 登录用户创建空间 |
| `list_my_spaces` | 当前用户创建的空间列表 |

后续需要扩展：编辑空间、关闭/重新激活、密码轮换、归档模板、驻留申请。

### 6.3 Guides

| Function | 作用 |
|---|---|
| `list_guides` | 攻略列表 |
| `list_cities` | 省下城市 |
| `list_districts` | 城市下区县 |
| `list_spots` | 区县下景点 |

后续需要扩展：攻略详情、用户投稿、管理员编辑器、图片/板块管理。

### 6.4 Admin

| Function | 作用 |
|---|---|
| `admin_stats` | 后台统计概览 |
| 其他 admin function | 待扩展空间/攻略/驻留审批 |

---

## 7. 认证与权限模型

### 7.1 登录态

- 使用 HttpOnly Cookie 保存 session token。
- 服务端根据 cookie 查询 `sessions` 和 `users`。
- 前端通过 `current_session` 决定显示登录入口或头像。

### 7.2 用户角色

| 角色 | 权限 |
|---|---|
| `user` | 创建和管理自己的空间、投稿内容 |
| `admin` | 管理空间、攻略、驻留申请 |
| `super_admin` | 最高权限，包含 admin 全部能力 |

### 7.3 权限原则

- “我的空间”只能显示当前用户自己的空间。
- 空间管理操作必须验证 `host_user_id == current_user.id` 或管理员权限。
- `/inspace/admin` 必须经过 admin/super_admin 检查。
- 私密空间访问和聊天必须检查 access session 与 password version。

---

## 8. 地图架构

地图产品原则：默认图层展示 **Space / 空间**，因为 Space 才是有坐标、有主理人、有生命周期、有扫码入口的真实地点数字入口。Guide 通过 Space 详情、攻略列表、搜索和后续可选攻略图层被发现。

### 8.1 地图加载

地图采用 MapLibre GL JS，但资源稳定性由 Rust 应用和 Nginx 保证：

- MapLibre JS/CSS 从本地 `/inspace/vendor/maplibre-gl/` 加载。
- OpenFreeMap 样式、瓦片、sprite、字体通过 `/inspace/ofm/` 代理。
- 禁止依赖 `unpkg` 或浏览器直接访问外部 tile 域名作为主路径。

### 8.2 Map UI 适配层

`crates/map-ui` 暴露 Rust 函数：

- `mount`
- `destroy`
- `sync_points`
- `focus_point`
- `set_style`
- `set_projection`
- `zoom_in / zoom_out`
- `enable_picker / disable_picker`

JS 细节保留在 `maplibre_shim.js` 中。Leptos 组件不直接操作 MapLibre 对象。

### 8.3 地图稳定性规则

- 页面切换时必须 `destroy` 地图实例。
- 重回首页必须重新 mount。
- 创建空间弹窗关闭时必须销毁弹窗地图。
- 新增地图能力必须补浏览器测试，避免再次出现“地图不加载”的回归。

---

## 9. 前端状态设计

| 状态 | 所在位置 | 说明 |
|---|---|---|
| 语言 | `i18n` context | 中英切换，页面级文案响应式更新 |
| 登录用户 | `current_session` Resource | Header、My Spaces、Admin 使用 |
| 空间列表 | `list_spaces` Resource | 首页搜索/筛选驱动 |
| 创建空间弹窗 | context signal | Header 和页面都可打开 |
| 地图实例 | `map-ui` 全局 store | 浏览器端 MapLibre 生命周期 |
| 私密验证 | server function + access session | 不应只存在前端内存 |

---

## 10. 当前已实现能力

- Rust workspace 分层。
- PostgreSQL schema 与种子数据。
- Axum + Leptos SSR + WASM hydration。
- `/inspace` 首页地图探索。
- 本地 MapLibre 资源和 OpenFreeMap 同源代理。
- 中英语言切换。
- 统一注册/登录。
- 用户头像入口。
- 创建空间弹窗和地图选点。
- 我的空间基础页面。
- `/inspace/admin` 基础后台。
- 私密空间密码边界与聊天密码版本规则。
- 攻略层级筛选基础能力。
- 浏览器测试覆盖地图、登录、创建弹窗、语言、移动端、导航回首页地图重挂载。

---

## 11. 后续实现路线

### Phase 1：稳定基础体验

- [ ] Header 根据当前路由正确高亮，而不是固定 Explore。
- [ ] 登录/注册成功后刷新全局 session 或跳转。
- [ ] 创建空间成功后刷新首页地图和我的空间列表。
- [ ] 增加退出登录。
- [ ] 创建空间表单补充类型、自定义类型、描述、标签、有效期。
- [ ] 优化移动端弹窗表单和地图高度。

### Phase 2：主理人空间管理

- [ ] 我的空间列表支持编辑。
- [ ] 空间密码轮换，更新 password version。
- [ ] 关闭、重新激活、删除或归档空间。
- [ ] 申请驻留。
- [ ] 空间模板化。

### Phase 3：攻略系统

- [ ] 攻略详情页。
- [ ] 结构化攻略编辑器。
- [ ] 用户投稿模式和管理员模式分离。
- [ ] 图片 URL / 上传策略。
- [ ] 攻略与空间的双向关联。

### Phase 4：实时互动

- [ ] WebSocket 聊天通道。
- [ ] 聊天消息持久化。
- [ ] 私密空间聊天权限校验。
- [ ] 求助模块。
- [ ] 简单游戏模块。

### Phase 5：后台运营

- [ ] 空间管理列表与详情。
- [ ] 攻略后台管理与编辑。
- [ ] 驻留申请审批。
- [ ] 用户管理。
- [ ] 操作审计日志。

### Phase 6：数据导入与生产化

- [ ] 完整导入旧 SQLite 中的空间、攻略、用户。
- [ ] 导入数据完整性校验。
- [ ] 定期任务处理空间过期。
- [ ] 统一错误格式与日志追踪。
- [ ] 数据库备份与恢复脚本。

---

## 12. 开发与部署命令

### 本地构建测试

```bash
cargo fmt --all
npm run build:wasm
DATABASE_URL=postgres://instant_space:instant_space_pass@127.0.0.1:5432/instant_space_rust cargo test --workspace
npm run test:map-runtime
npm run test:browser
```

### 生产部署

```bash
npm run build:wasm
cargo build -p instant-space-app --release
systemctl restart instant-space-rust.service
nginx -t && systemctl reload nginx
```

生产入口：

```text
https://opctoai.com/inspace
```

服务约定：

```text
instant-space-rust.service -> 127.0.0.1:3001
Nginx /inspace          -> 127.0.0.1:3001
Nginx /inspace/ofm      -> OpenFreeMap proxy
Nginx /inspace/vendor   -> app vendor assets
```

---

## 13. 架构决策记录

### ADR-001：使用 PostgreSQL 而不是 SQLite

旧系统可用 SQLite 快速开发，但 Rust 版面向线上长期运行。PostgreSQL 更适合并发、索引、JSONB、权限边界和后续实时功能。

### ADR-002：使用 Leptos Server Functions

当前产品仍在快速变化阶段。Server Functions 能减少前后端 DTO 重复，并让 Rust 类型直接贯穿页面和服务端逻辑。等外部 API 稳定后，再补 REST/JSON API。

### ADR-003：地图资源本地化和同源代理

地图是首页核心能力，不允许因为 CDN、跨域、缓存或外部域名不稳定导致首页不可用。MapLibre 运行时本地化，瓦片走 `/inspace/ofm`。

### ADR-004：Admin 放在 `/inspace/admin`

线上主路径是 `/inspace`，后台也必须在同一产品路径下。裸 `/admin` 仅做兼容重定向。

### ADR-005：创建空间使用弹窗而非页面

用户创建空间的关键上下文是地图位置。弹窗能保持探索场景，不打断首页地图，同时支持独立“我的空间”页面承载管理能力。

---

## 14. 防回归要求

每次修改以下能力必须补或更新测试：

- 地图加载、样式切换、页面返回重挂载。
- 创建空间弹窗与地图选点。
- 登录态 Header 显示。
- `/inspace/my-spaces` 权限和列表。
- `/inspace/admin` 权限。
- 攻略层级筛选。
- 私密空间验证与密码版本。

任何影响 `/inspace` 首页的改动，部署前必须至少通过：

```bash
npm run test:browser
npm run test:map-runtime
```

---

## 15. 术语对照

| 中文 | 英文/代码概念 |
|---|---|
| 空间 | Space |
| 主理人 | Host / owner |
| 私密空间 | Private Space |
| 访问验证 | Access Session |
| 攻略 | Guide |
| 驻留 | Resident |
| 模板 | Template |
| 后台 | Admin |
| 地图选点 | Coordinate Picker |

---

## 16. 非目标

- 不保持旧 Next.js API 路径逐字兼容，除非未来需要对外开放 REST API。
- 不继续依赖 Node.js 作为线上运行时。
- 不把旧 Prisma schema 原样搬到 Rust；只保留必要业务含义。
- 不把管理员入口放在首页主内容中。
- 不在前端内存中长期模拟聊天、求助、游戏数据。
