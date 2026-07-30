# inspace / SpaceOS 网站模块架构与 Phase 缺口审计

> 日期：2026-07-27
>
> 用途：当页面或功能出现 Bug 时，先定位模块，再修改该模块拥有的页面、服务、领域模型、数据库和测试。
> 本文是“维护导航图”，不是新的产品宣传文案，也不代表本轮要立即实现所有缺口。

---

## 1. 为什么需要这份文档

当前仓库已经不是早期的“地图 + 攻略”原型，而是一个包含 SSR、WASM、地图、空间管理、攻略、实时聊天、在场证明、地点记忆、胶囊和后台编辑器的全栈系统。

但现有文档存在三个问题：

1. `docs/PHASES.md` 同时保留了早期差距和后续落地记录，前面的“未实现”有些已经在后文实现。
2. 白皮书的 Phase 1–3 是战略路线；工程文档的 Phase 1–7 是交付路线，两套编号含义不同。
3. 一些逻辑模块横跨多个文件。只看页面文件修 Bug，容易漏掉 Server Function、repository、迁移、WASM 或 Nginx。

因此以后维护采用两层视图：

```text
战略路线：白皮书 Phase 1–3
  └── 回答产品最终往哪里走

交付路线：Rust 工程 Phase 1–7
  └── 回答当前网站完成到哪里

维护模块：M00–M24 / F01–F06
  └── 回答 Bug 应该修哪里
```

---

## 2. 当前系统事实基线

### 2.1 技术栈

```text
浏览器 UI       Leptos SSR + WASM Hydration
应用服务         Axum
实时连接         Axum WebSocket + Tokio broadcast
数据库           PostgreSQL + sqlx migrations
地图             MapLibre GL JS v5 + Rust/WASM shim
认证             Argon2 + HttpOnly Session Cookie
部署             systemd + Nginx + Cloudflare
线上前缀         /inspace
```

### 2.2 当前生产数据快照

2026-07-27 只读检查：

| 对象 | 数量 | 说明 |
|---|---:|---|
| Users | 44 | 已有真实登录与角色数据 |
| Spaces | 2,714 | 其中 active 2,702；新增 1,700 个待认领中国省级空间 |
| Guides | 2,701 | 新增 1,700 份与待认领空间一一对应的基础攻略 |
| Chat messages | 49 | 实时聊天已产生持久化数据 |
| Space traces | 8 | 地点留言/痕迹已投入使用 |
| Space capsules | 7 | 已有双锁打开与双现场证明写入数据 |
| Helps | 0 | 只有表结构，没有产品模块 |
| Games | 0 | 只有表结构，没有产品模块 |
| Space members | 0 | 只有表结构，成员系统未产品化 |
| Space templates | 0 | 现有“归档模板”只改 Space 状态，没有写模板表 |
| Homepage versions | 6 | 首页 CMS 草稿/发布版本已工作 |

### 2.3 已修复的生产结构漂移

生产库曾出现 `_sqlx_migrations` 记录 `20260712001000 admin audit log` 成功，但实际缺少 `admin_audit_log` 表的漂移。2026-07-27 已完成修复：

- 新增幂等修复迁移 `20260727000200_repair_admin_audit_log.sql`；
- 恢复 `admin_audit_log` 表、主键及 `created_at` / `actor_id` 索引；
- 管理后台不再用 `unwrap_or_default()` 吞掉审计查询错误，而是显示明确错误状态；
- `crates/db/build.rs` 监听整个 `migrations` 目录，避免新增 SQL 文件未触发嵌入式 migrator 重编译；
- 生产应用角色已获得对 `users` 表的最小 `REFERENCES` 权限，使 `actor_id` 外键迁移可以执行。

以后排查迁移问题必须同时核对 `_sqlx_migrations`、真实表/列/索引和服务启动日志，不能只相信迁移记录。

---

## 3. Phase 缺口审计

## 3.1 Rust 工程交付路线 Phase 1–7

状态定义：

- **已实现**：主闭环已经存在，并有代码/数据证据。
- **部分实现**：可以使用，但仍缺关键体验、权限、扩展或工程完整性。
- **未实现**：只有规划、表结构或文案，没有可用产品闭环。

| 工程 Phase | 当前状态 | 已有能力 | 仍未实现/不完整 |
|---|---|---|---|
| Phase 1 用户主闭环 | 已实现 | 注册、登录、退出、全局会话刷新、创建空间、地图出现、我的空间、导航与中英切换 | 创建后跨页面刷新仍需持续回归；错误反馈缺统一模型 |
| Phase 2 主理人空间管理 | 部分实现 | 编辑空间、关闭、重开、删除、重置密码、驻留申请、状态归档 | 完整描述/标签/有效期管理不一致；模板表未使用；商家认领不是同一能力；缺统一操作确认与审计 |
| Phase 3 私密空间与访问 | 部分实现 | 密码验证、`access_sessions`、密码版本、私密聊天校验、现场 Wi-Fi 口令 | 没有成熟的权限中心；QR/GPS/Discord 证明强度不一致；成员角色系统未落地 |
| Phase 4 攻略系统 | 基本实现 | 分类分页、详情、结构化编辑、绑定/解绑空间、更新同一攻略、删除/归档、用户与管理员权限 | 图片仍以 URL 为主；无媒体上传/资产库；缺协作修订、内容版本、AI 辅助整理 |
| Phase 5 后台运营 | 部分实现 | 概览、首页编辑器、空间、攻略、驻留、用户角色、审计日志、主标题显式断行 | 危险操作确认不统一；缺服务端分页/导出；后台仍缺完整运营工作流 |
| Phase 6 实时互动 | 部分实现 | 独立聊天页、WebSocket 广播、历史持久化、在线人数、私密版本校验、重连 | 无消息类型、求助、通知、成员列表、送达状态、房间限流和连接上限；游戏未实现 |
| Phase 7 数据迁移与生产化 | 部分实现 | 启动迁移、健康/就绪检查、空间过期任务、部署脚本、测试集合、GeoNames/种子导入 | `crates/importer` 只统计旧 SQLite，没有实际导入；无 CI；无正式备份保留/恢复演练；无统一错误结构；迁移漂移检测不足 |

### 3.1.1 当前最重要的工程缺口

按风险排序：

1. **数据库迁移漂移**：迁移显示成功但表缺失，说明不能只信 `_sqlx_migrations`。
2. **错误被默认值吞掉**：部分资源用 `unwrap_or_default()`，数据库错误可能伪装成“空数据”。
3. **Help/Game/Member/Template 是空壳**：有表不等于有模块。
4. **Importer 没有真正迁移**：当前只读取并输出数量。
5. **首页 CMS 内容模型不完整**：标题只能输入纯文本，缺显式换行/断句控制；也没有媒体区块和区块级内容版本。
6. **CSS 物理边界过大**：`ui-system.css`、`main.css`、`inspace-world.css` 都超过 140 KB，多个页面共享级联，容易发生跨页面覆盖。
7. **部分列表仍是客户端管理**：当数据继续增长时需要服务端分页、搜索和筛选成为统一契约。

---

## 3.2 白皮书战略路线 Phase 1–3

白皮书 Phase 与工程 Phase 不是同一套编号。

### 白皮书 Phase 1：占领心智与头部节点

目标：核心旅游城市、地标与店铺；主打“时空记忆”和“在场聊天”。

**当前完成度：约半完成。**

已经具备：

- 全球地图、国家/城市/地点数据；
- 2,700+ 空间和对应攻略，其中 1,700 个中国省级空间标记为待认领；
- 空间详情与攻略；
- 独立聊天页和 WebSocket；
- 在线人数；
- 留言/痕迹、空间纪事统计；
- 地点胶囊：打开需要现场 Wi-Fi 口令 + 作者私下口令；写入需要现场 Wi-Fi 口令 + GPS 半径验证；
- QR、GPS、现场口令、Discord 等在场证明入口。

尚未完成：

- 真正的历史时间轴与多媒体“数字地层”；
- 照片、录音、视频等地点记忆资产；
- 锚点视频；
- 严格的“在场聊天室”模式（当前公共空间允许远程进入）；
- 在线成员身份、主持人和角色；
- 活动、求助和地点游戏；
- 3D 空间主页、实况视频、天气渲染、氛围音乐；
- AI 历史讲解员、空间总结者和虚拟店员。

### 白皮书 Phase 2：主理人与商业生态

目标：商家/创作者认领，打通线下拉新、空间互动和商业变现。

**当前完成度：早期基础。**

已经具备：

- 用户可创建并管理空间；
- 主理人权限边界、公开招募页与待认领内容标记；
- 驻留申请与管理员审批；
- QR 分享；
- 空间密码和现场口令。

尚未完成：

- 认领已有现实商家的 SpaceID；
- 商家身份与地址证明；
- 官方空间标识和主理权转移；
- 餐饮点单；
- 优惠券与会员；
- 排队与预约；
- 票务或付费权限；
- 空间广告、AR 广告位；
- 商业数据报表和结算；
- 模板市场及真正的 `space_templates` 数据闭环。

### 白皮书 Phase 3：全场景介观网络 / Meso Web

目标：开放平台、智能硬件、协议化和设备无关。

**当前状态：基本未实现。**

尚未完成：

- 稳定的公开 REST/Open API；
- 开发者账号、API Key、OAuth 与配额；
- SpaceID 公共协议；
- Webhook；
- JavaScript/Rust/移动端 SDK；
- AR 游戏开发接口；
- NFC、蓝牙、IoT、车机、AR 眼镜接入；
- IPFS/分布式存储；
- 用户数据导出、资产确权与可迁移性；
- 多租户、第三方扩展和插件市场。

---

## 4. 白皮书功能模块完成度

| 白皮书能力 | 状态 | 当前对应模块 | 缺口 |
|---|---|---|---|
| 空间主页 | 部分实现 | M09 | 有详情和攻略，但没有 3D、视频、天气和氛围 |
| 历史时间轴 | 部分实现 | M14 | 有 traces/chronicle，没有真正时间轴和媒体 |
| 空间聊天室 | 已实现基础 | M13 | 缺成员、角色、通知、限流与严格在场模式 |
| AI 驻场智能体 | 未实现 | F02 | 无模型、知识库、权限和成本控制 |
| 虚拟留言墙 | 已实现基础 | M14 | 当前是文字留言，不是可装修的贴纸墙/同心锁墙 |
| 空间活动 | 未实现 | F03 | 无活动模型、日程和报名 |
| 求助 | 未实现 | F03 | `helps` 只有空表 |
| 游戏 | 未实现 | F03 | `games` 只有空表 |
| Proof of Location | 部分实现 | M10 | 现场口令最强；QR/GPS/Discord 仍需更明确的信任等级 |
| 时空漂流瓶/胶囊 | 已实现核心 | M15 | 目前是文字；缺照片/录音、通知和领取人关系 |
| 同频偶遇 | 未实现 | F02/F03 | 无轨迹画像、匹配、隐私与安全机制 |
| 锚点视频 | 未实现 | F01 | 无媒体资产系统 |
| 商家认领 | 未实现 | F04 | 创建空间不等于认领已有地点 |
| 点单/优惠券/排队 | 未实现 | F04 | 无商业服务模型 |
| 空间广告 | 未实现 | F04 | 无广告位、审核、计费 |
| Open API/SDK | 未实现 | F05 | 当前主要是 Leptos Server Functions |
| 智能硬件 | 未实现 | F05 | 无设备身份和消息协议 |
| IPFS/数据确权 | 未实现 | F06 | 当前数据集中存储在 PostgreSQL |

---

## 5. 总体架构图

```text
┌──────────────────────────────── 浏览器 ────────────────────────────────┐
│ M02 全局导航  M03 首页  M05 探索  M06 地图  M09 空间详情  M13 聊天      │
│ M11 攻略      M08 工作台 M17 后台  M14 记忆  M15 胶囊                 │
│                    Leptos SSR + WASM Hydration                        │
└───────────────────────────────┬────────────────────────────────────────┘
                                │ Leptos Server Functions / WebSocket
┌───────────────────────────────▼────────────────────────────────────────┐
│ M00 Axum Runtime / Routing                                             │
│ M01 Auth  M07 Spaces  M12 Guides  M10 Access  M13 Realtime             │
│ M17 Admin M18 Geo     M04 Site CMS  M14/M15 Traces                     │
└───────────────────────────────┬────────────────────────────────────────┘
                                │ sqlx repositories
┌───────────────────────────────▼────────────────────────────────────────┐
│ M20 Domain + DB                                                        │
│ users / sessions / spaces / guides / access_sessions / chat_messages   │
│ space_traces / space_capsules / site_page_configs / geo_places         │
└───────────────────────────────┬────────────────────────────────────────┘
                                │
┌───────────────────────────────▼────────────────────────────────────────┐
│ M21 生产运行：systemd / Nginx / Cloudflare / migrations / health        │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 6. 当前维护模块 M00–M24

## M00 — 应用启动、路由与 SSR Shell

**负责**：进程启动、数据库迁移、Axum Router、Leptos 路由、静态资源、SSR/WASM 注入。

- 入口：`app/src/main.rs`
- 根 UI：`app/src/app.rs`
- WASM 初始化：`app/src/lib.rs`
- 典型 Bug：404、页面路由错、WASM 不加载、静态资源路径错、服务启动失败。
- 第一检查：`/health`、`/ready`、HTML 中 WASM 文件名、systemd 日志。
- 回归：`tests/build/wasm-build.mjs`、`tests/browser/hydration-assets.spec.ts`

## M01 — 认证、会话与角色

**负责**：注册、登录、退出、当前用户、Session Cookie、admin/super_admin 权限。

- UI：`app/src/pages/auth.rs`
- Server：`app/src/server/auth.rs`
- Domain：`crates/domain/src/auth.rs`
- DB：`crates/db/src/users.rs`
- Crypto：`crates/auth/src/lib.rs`
- 表：`users`、`sessions`
- 典型 Bug：登录成功但导航没变、退出无效、权限误判、Session 过期。
- 第一检查：`current_session()` 与 `instant_session` Cookie；不要先改 Header CSS。
- 测试：`tests/browser/auth.spec.ts`、`tests/browser/private-chat.spec.ts`

## M02 — 全局导航、应用壳与国际化

**负责**：桌面侧栏、手机抽屉、底部导航、语言、用户/后台入口、路由高亮。

- UI：`app/src/components/header.rs`
- 状态：`app/src/app_state.rs`
- i18n：`app/src/i18n.rs`
- 样式：`app/style/app-shell.css`
- 典型 Bug：按钮能点但跳错、菜单遮挡、切语言卡顿、后台入口不见。
- 第一检查：当前 pathname、`/inspace` 前缀、WASM hydration 是否成功。
- 测试：`tests/browser/navigation-controls.spec.ts`

## M03 — 公共首页呈现

**负责**：已发布首页内容、首页区块顺序、首页视觉叙事和动效。

- 页面：`app/src/pages/home.rs`
- 内容模型：`crates/domain/src/site.rs`
- 动效：`app/src/field_parallax.js`
- 样式：`app/style/song-system.css`、`app/style/inspace-world.css`
- 典型 Bug：主标题遮挡、中文断句错误、iPad 版式错、首页显示草稿而非发布版。
- 第一检查：`get_public_home_config()` 返回的 published config。
- 与 M04 区别：M03 是访客看到的首页；M04 是管理员编辑工具。

## M04 — 首页 CMS / 页面编辑器

**负责**：草稿、发布、版本恢复、结构树、画布、属性面板、首页主题与 SEO 内容。

- 页面：`app/src/pages/admin_home.rs`
- Server：`app/src/server/site.rs`
- Domain：`crates/domain/src/site.rs`
- DB：`crates/db/src/site.rs`
- 表：`site_page_configs`、`site_page_versions`
- 样式：`app/style/backoffice.css`
- 主标题已支持显式换行：管理员在 textarea 中按 Enter，草稿、画布预览和公共首页都会保留 `\n`，渲染使用 `white-space: pre-line`，不硬编码 `<br>`。
- 当前剩余缺口：尚未支持桌面端与移动端分别配置不同断句；如果以后需要独立断句，再扩展 `HomeHeroConfig` 内容模型。
- 典型 Bug：编辑器能改但首页不变、保存与发布混淆、版本恢复失败、预览与线上不一致。

## M05 — 探索空间与分类分页

**负责**：空间搜索、分类、分页、列表/网格发现。

- 页面：`app/src/pages/explore.rs`
- 组件：`app/src/components/guide_browser.rs`（共享浏览控件时注意命名）
- Server：`app/src/server/spaces.rs::list_space_page`
- DB：`crates/db/src/spaces.rs::list_home_spaces_page`
- 典型 Bug：1 万条数据卡顿、分页重复、筛选不生效、地图与列表数量不一致。
- 第一检查：服务端 `limit/offset/total`，不要先做客户端隐藏。

## M06 — 地图运行时

**负责**：MapLibre 懒加载、地图样式、2D/3D 投影、聚合点、marker、地图选点。

- 页面/组件：`app/src/components/map_workspace.rs`、`map_home.rs`
- Bootstrap：`app/src/map_boot.js`
- Rust/WASM API：`crates/map-ui/src/lib.rs`
- JS Adapter：`crates/map-ui/src/maplibre_shim.js`
- 样式：`app/style/main.css`、`app/style/app-shell.css`
- Nginx：`/inspace/ofm`、`/inspace/vendor`
- 典型 Bug：一直加载、从首页跳地图不挂载、瓦片 404、marker 不刷新、手机控件遮挡。
- 定位顺序：DOM `#map` → loader → MapLibre store → style loaded → tile requests → marker data。
- 测试：`map-health.mjs`、`map-marker-check.mjs`、`map-runtime-contract.mjs`

## M07 — 创建空间

**负责**：创建弹窗、地图选点、地理反查、自动现场口令、创建后刷新。

- UI：`app/src/components/space_form.rs`
- Server：`app/src/server/spaces.rs::create_space`
- DB：`crates/db/src/spaces.rs::create_host_space`
- Geo：M18
- 表：`spaces`
- 典型 Bug：创建按钮没反应、点位没保存、地图出现但列表没有、密码未返回。
- 第一检查：Server Function 返回值和 `refresh_spaces()` 信号。

## M08 — 用户工作台与空间生命周期

**负责**：我的空间、搜索分页、编辑、关闭、重开、删除、驻留申请、重置密码、攻略入口。

- 页面：`app/src/pages/host.rs`
- Server：`app/src/server/spaces.rs`
- DB：`crates/db/src/spaces.rs`
- 样式：`app/style/workspace.css`
- 典型 Bug：管理按钮无效、编辑别人空间、空间状态和地图不一致、攻略入口找不到。
- 已知结构债：`archive_template()` 只把 Space 状态改成 `template`，没有写 `space_templates`。

## M09 — 空间详情聚合页

**负责**：一个 Space 的基础资料、攻略、分享、记忆、胶囊、讨论入口。

- 页面：`app/src/pages/space.rs`
- 组件：`space_detail.rs`、`space_share.rs`、`space_traces.rs`
- Server：M07/M10/M12/M14/M15 的聚合
- 典型 Bug：空间详情很乱、某一模块不显示、聊天入口错、手机滚动被锁。
- 修复原则：先判断是聚合布局问题，还是子模块数据问题；不要把所有逻辑继续堆进 `space.rs`。

## M10 — 私密访问与在场证明

**负责**：空间密码、访问 Session、密码版本、GPS/QR/Discord/现场口令证明。

- UI：`private_verify.rs`、`presence.rs`
- Server：`app/src/server/chat.rs`、`app/src/server/traces.rs`
- DB：`crates/db/src/chat.rs`、`crates/db/src/traces.rs`
- 表：`access_sessions`
- 典型 Bug：私密空间无限验证、改密码后旧连接未失效、现场口令正确却不通过。
- 信任规则：现场口令由 Argon2 服务端校验；GPS/QR/Discord 不能等同于强安全凭证。

## M11 — 攻略浏览与详情

**负责**：攻略分类、分页、详情、结构化 sections、关联空间入口。

- 页面：`app/src/pages/guides.rs`
- 组件：`app/src/components/guide_browser.rs`
- Server：`get_guide_detail`、`list_guide_page`
- DB：`crates/db/src/guides.rs`
- 典型 Bug：攻略打不开、分页错、sections 丢失、空间关联不显示。

## M12 — 攻略编辑、所有权与绑定

**负责**：新建/更新攻略、草稿/发布/归档、绑定空间、权限和删除。

- 页面：`app/src/pages/guides.rs::GuideEditorPage`
- Server：`app/src/server/guides.rs`
- DB：`crates/db/src/guides.rs`
- Domain：`crates/domain/src/guides.rs`
- 典型 Bug：重复保存生成新攻略、删不掉、用户能改别人攻略、绑定空间错误。
- 第一检查：页面是否带 `guide_id`/`space_id`，服务端走 update 还是 create。
- 测试：`delete-guide-test.mjs`、`guide-admin.spec.ts`

## M13 — 实时聊天

**负责**：独立聊天页、WebSocket 房间、历史、在线人数、重连和私密版本校验。

- 页面：`app/src/pages/space.rs::SpaceChatPage`
- 浏览器运行时：`app/src/chat_realtime.js`
- WebSocket：`app/src/realtime.rs`
- Server fallback：`app/src/server/chat.rs`
- DB：`crates/db/src/chat.rs`
- 表：`chat_messages`
- Nginx：`docs/NGINX_WEBSOCKET.md`
- 典型 Bug：聊天连不上、消息发出收不到、在线人数不归零、Safari 重连循环。
- 定位顺序：WS URL → Nginx Upgrade → access session → room join → DB insert → broadcast。

## M14 — 地点记忆、留言墙与空间纪事

**负责**：留言/痕迹、在场标签、分页、删除/隐藏、地点沉淀统计。

- UI：`app/src/components/space_traces.rs`
- Presence：`app/src/components/presence.rs`
- Server：`list_traces`、`leave_trace`、`hide_trace`
- DB：`crates/db/src/traces.rs`
- Domain：`crates/domain/src/traces.rs`
- 表：`space_traces`
- 典型 Bug：留言后不刷新、现场证明标签错、分页总数错、用户删除权限错。
- 长期缺口：媒体、贴纸墙、同心锁的专用内容类型和真正的历史时间轴。

## M15 — 地点胶囊 / 漂流瓶

**负责**：封存胶囊、收件提示、日期锁、现场口令锁、作者口令锁、距离反馈和防暴力猜测。

- UI：`app/src/components/space_traces.rs`
- Server：`seal_capsule`、`open_capsule`
- DB：`crates/db/src/traces.rs`
- Domain：`CapsuleSummary`、`CapsuleOpenResult`、`CapsuleSealResult`
- 表：`space_capsules`
- 典型 Bug：在地点仍打不开、口令校验顺序错、失败次数错误、已打开状态不更新。
- 核心规则：打开必须同时满足“现场 Wi-Fi 口令”和“胶囊作者口令”；埋下必须同时满足“现场 Wi-Fi 口令”和“GPS 在所选半径内”。QR 不能替代写入或打开的硬验证。

## M16 — 分享链接与二维码

**负责**：稳定 Space URL、复制、二维码、扫码进入参数。

- UI：`app/src/components/space_share.rs`
- Presence 接入：`presence.rs::detect_scan`
- 典型 Bug：二维码落错页面、复制前缀不对、扫码后未识别。
- 风险：当前二维码依赖外部 QR 图片服务；后续应本地生成并支持下载物料。

## M17 — 管理控制台

**负责**：概览、首页 CMS、空间、攻略、驻留、用户、角色、审计。

- 导航：`app/src/components/admin_nav.rs`
- 页面：`app/src/pages/admin*.rs`
- Server：`app/src/server/admin.rs`，并复用 spaces/guides/site
- DB：`crates/db/src/admin.rs`
- 表：`admin_audit_log`（生产已由 `20260727000200_repair_admin_audit_log.sql` 恢复）
- 样式：`app/style/backoffice.css`
- 典型 Bug：普通用户看见后台、按钮无效、操作后无反馈、审计为空、首页发布不生效。
- 第一检查：`require_admin_user()`、super_admin 额外边界、生产表是否存在。

## M18 — 地理数据与反向定位

**负责**：国家/省州/城市/区县、最近地点、首都数据、地图选点反查。

- Server：`app/src/server/geo.rs`
- DB：`crates/db/src/geo.rs`、`locations.rs`
- Bootstrap：`app/src/geo_capitals_boot.js`
- 数据：`geo_places`、`geo_capitals`、`locations`
- 导入：`scripts/import-geonames.py`
- 典型 Bug：选点城市错误、国家筛选为空、经纬度中心错误。

## M19 — SEO、robots 与 sitemap

**负责**：标题、描述、robots、动态 sitemap、可索引空间/攻略。

- Meta：`app/src/pages/home.rs`、各详情页
- Runtime：`app/src/main.rs::robots_txt/sitemap_xml`
- CMS：M04 SEO 面板
- 典型 Bug：Google 不展示、URL 未进 sitemap、后台被索引、标题仍是旧文案。
- 注意：SEO 与 GEO/生成式搜索优化是内容和结构问题，不等于 M18 地理数据。

## M20 — 领域模型、Repository 与迁移

**负责**：业务类型、权限边界、SQL、数据转换、schema。

- Domain：`crates/domain/src/*.rs`
- DB：`crates/db/src/*.rs`
- Migrations：`crates/db/migrations/*.sql`
- 典型 Bug：页面修好但数据错误、枚举不兼容、迁移成功记录和真实 schema 不一致。
- 原则：页面不能自己复制业务规则；权限必须在 Server Function 再校验。

## M21 — 部署、缓存、健康检查与可观测性

**负责**：release/WASM 构建、systemd、Nginx、Cloudflare 缓存、健康、就绪、日志和备份。

- 构建：`scripts/build-wasm.mjs`
- 部署：`scripts/deploy.sh`
- Runtime：`app/src/main.rs`
- 服务：`instant-space-rust.service`
- Nginx：`/etc/nginx/conf.d/opctoai.com.conf`
- 典型 Bug：代码改了浏览器仍旧版、服务重启后旧二进制、CSS 缓存、WASM 404、数据库迁移漂移。
- 第一检查：生产实际执行文件 `/usr/local/bin/instant-space-app`，不要只看 `target/release`。

## M22 — 数据导入、种子与内容初始化

**负责**：GeoNames、批量空间/攻略、旧 SQLite 迁移、幂等导入和数据校验。

- Rust importer：`crates/importer`
- Geo：`scripts/import-geonames.py`
- 批量种子：`scripts/seed/*`
- 当前缺口：Rust importer 只输出源库数量，不向 PostgreSQL 写入。
- 典型 Bug：重复导入、空间有攻略但关联错、国家数量不对、导入后地图无点。

## M23 — 视觉系统、响应式与动效

**负责**：宋式空间系统、tokens、排版、断点、动效与无障碍。

- 现行层：`song-system.css`、`app-shell.css`、`workspace.css`、`backoffice.css`
- 遗留大层：`ui-system.css`、`main.css`、`inspace-world.css`
- JS 动效：`field_parallax.js`
- 典型 Bug：一个页面改 CSS 导致另一个页面坏、iPad 标题竖排、半透明文字、固定底栏遮挡。
- 原则：先确认规则属于哪个模块；禁止继续无边界地追加全站 `!important`。

## M24 — 自动化测试与回归证据

**负责**：构建契约、权限、浏览器流程、地图、聊天、胶囊、响应式和视觉回归。

- Browser：`tests/browser/*`
- Build：`tests/build/*`
- 截图：`output/playwright/*`
- 典型 Bug：功能已坏但测试仍绿、测试硬编码旧版本号、测试和构建并发导致 WASM 临时 404。
- 原则：构建和浏览器回归不要同时修改/读取 `target/site/pkg`。

## M25 — 关于 inspace 与空间主理人招募

**负责**：解释产品使命、介观空间定位、精简创始人寄语和公开主理人招募。

- 页面：`app/src/pages/about.rs`
- 路由：`/inspace/about`
- 样式：`app/style/about.css`
- 入口：全局侧栏底部、首页主理人区
- 内容种子：`scripts/seed/seed_china_provinces.py` 为 34 个省级地区建立待认领基础空间与攻略
- 当前边界：这是招募和内容标记，不是正式认领工作流；真实商家/创作者确权仍属于 F04。
- 典型 Bug：招募文案存在但没有入口、About 在手机溢出、待认领空间被错误绑定给系统管理员。

---

## 7. 未来模块 F01–F06

这些模块来自白皮书，目前不应继续塞进现有大页面。

| 编号 | 未来模块 | 建议独立边界 |
|---|---|---|
| F01 | 媒体与数字地层 | media assets、上传、转码、照片/音频/视频、时间轴 |
| F02 | AI 介观智能体 | AI provider、Space 知识库、权限、成本、审核、摘要 |
| F03 | 活动/求助/游戏/同频偶遇 | activities、helps、games、notifications、safety |
| F04 | 商家认领与商业服务 | claims、merchant verification、coupon、queue、reservation、ads |
| F05 | 开放平台和设备接入 | public API、API keys、OAuth、webhooks、SDK、device registry |
| F06 | 数据确权与分布式存储 | export、ownership、content address、IPFS adapter、retention |

---

## 8. Bug 症状 → 模块定位速查

| 症状 | 首查模块 | 第一批文件 |
|---|---|---|
| 首页主标题不能手动分行 | M04 + M03 | `domain/site.rs`、`admin_home.rs`、`home.rs` |
| 刷新仍是旧页面 | M21 | WASM/CSS 版本、Nginx sub_filter、Cloudflare cache |
| 菜单能用，其他按钮失效 | M00/M02 | hydration、`app/src/lib.rs`、`header.rs` |
| 地图一直加载 | M06 | `map_boot.js`、`maplibre_shim.js`、瓦片请求 |
| 创建空间后地图没点 | M07 + M06 | create 返回、刷新信号、marker 数据 |
| 我的空间按钮无效 | M08 | `host.rs` Action、spaces Server Function |
| 空间详情手机不能滚 | M09 + M23 | `space.rs`、overflow、固定底栏让位 |
| 私密空间反复要密码 | M10 | access session Cookie、password_version |
| 攻略删不掉/重复生成 | M12 | guide_id、create/update 分支、owner 校验 |
| 聊天连不上 | M13 + M21 | WS URL、Nginx Upgrade、access session |
| 留言显示成远程 | M10 + M14 | PresenceClaim、现场口令/GPS 判定 |
| 胶囊到场仍打不开 | M15 + M10 | onsite code、passphrase、failed attempts |
| 后台审计一直为空 | M17 + M20 | `admin_audit_log` 是否存在、错误是否被吞 |
| 分类分页数据重复 | M05/M11 | offset、total、稳定排序 |
| iPad 文字一字一行 | M23 | 有效画布宽度、标题最大行长、grid min-width |
| Google 不收录空间 | M19 | robots、sitemap、meta、公开状态 |
| 国家/城市定位错误 | M18 | reverse geo、geo_places 数据 |

---

## 9. 每次修 Bug 的标准流程

```text
1. 记录 Route、账号角色、设备宽度、操作步骤
2. 根据上表选择一个主模块
3. 先复现，再查看：UI → Server Function → Domain → DB → Infra
4. 只修改主模块拥有的文件；跨模块时明确列出依赖模块
5. 增加该模块的最小回归测试
6. 构建与浏览器测试串行执行，避免 target/site/pkg 被并发删除
7. 部署后验证线上真实资源版本
8. 把长期架构事实写回 memory.md 或本文
```

### Bug 报告模板

```text
标题：[Mxx] 简短症状
路由：/inspace/...
角色：访客 / 用户 / 主理人 / admin / super_admin
设备：桌面 / iPad 横屏 / iPad 竖屏 / 手机
步骤：1... 2... 3...
实际：...
预期：...
Console：...
Network：...
关联数据 ID：space_id / guide_id / capsule_id
首次怀疑模块：Mxx
```

---

## 10. 模块化改造原则（以后逐步执行，不在本轮实施）

1. **一个业务模块拥有一条完整竖切片**：页面、组件、Server Function、Domain、DB、测试一起归档。
2. **不要按“前端/后端”分 Bug**：例如胶囊是 M15，不是“前端胶囊”和“后端胶囊”两个无人负责的系统。
3. **共享壳与业务页面分离**：M02 不能持有 Space/Guide 业务规则。
4. **数据库错误不能变成空数组**：重要后台和管理页面应显示可诊断错误。
5. **未来功能先建模块再建表**：避免再次出现 `helps`、`games`、`space_templates` 只有表结构的空壳。
6. **CSS 归属必须明确**：页面样式优先进入对应模块 CSS；全站层只放 tokens、reset、壳和真正共享组件。
7. **公开 API 与内部 Server Function 分开**：白皮书 Phase 3 的 Open API 不能直接暴露当前内部函数。
8. **权限只信服务端**：UI 隐藏按钮不是权限控制。
9. **迁移完成必须检查真实 schema**：不仅检查 `_sqlx_migrations` 记录。
10. **功能完成必须有生产证据**：代码、表、数据、浏览器行为四者至少三者一致。

---

## 11. 下一阶段建议（仅规划，不执行）

### P0 工程安全

1. ✅ 已修复并记录 `admin_audit_log` 迁移漂移。
2. ✅ 审计日志资源已取消无声 `unwrap_or_default()`；其他关键后台资源仍需逐项审计。
3. 增加 schema contract 检查：关键表、列、索引启动时或 CI 验证。
4. 建立 CI，至少串行运行 Rust check、WASM 构建契约和关键浏览器烟测。

### P1 完成白皮书 Phase 1

1. 把 M14 从文字留言升级为真正“地点记忆时间轴”。
2. 建立 F01 媒体资产模块，支持照片、录音、视频。
3. 给 M13 增加“仅在场聊天室”模式、成员列表和角色。
4. 建立 F03 求助/活动，而不是继续在聊天正文里模拟。
5. 为 M04 增加桌面端/移动端独立断句（基础显式换行已完成）。

### P2 开始白皮书 Phase 2

1. 先设计商家/创作者认领，不要把“创建空间”当“认领”。
2. 设计官方 SpaceID、主理权转移与证明材料。
3. 商业功能从预约/排队等一个真实闭环开始，不同时做点单、会员、广告。

### P3 长期协议化

先稳定内部领域模型和公开 ID，再设计 Open API/SDK；不要直接把 Leptos Server Functions 当公共协议。

---

## 12. 文档优先级

以后遇到冲突时，按以下顺序判断：

1. 白皮书和创始人寄语：长期使命和不可丢失的产品灵魂。
2. `docs/PRODUCT_VISION.md`：当前阶段用户闭环。
3. 本文：当前模块边界和 Bug 定位。
4. `docs/PHASES.md`：历史执行记录，必须结合日期阅读。
5. `docs/ARCHITECTURE_RUST.md`：Rust 初始架构基线，其中“当前状态”可能已经过期。
6. `memory.md`：近期部署、问题根因和实际 QA 证据。
