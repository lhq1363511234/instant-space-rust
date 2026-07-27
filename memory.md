# Instant Space Rust — Agent Memory

> 给后续模型/会话快速接盘用。有更新就往下追加，不要删历史关键结论。

## 项目

- 路径：`/root/opt/instant-space-rust`
- 产品：Instant Space（Rust/Leptos 重写）
- 本地：http://127.0.0.1:3001
- 线上：opctoai.com `/inspace`（Nginx → `instant-space-rust.service` → release 二进制）
- 服务：`/etc/systemd/system/instant-space-rust.service`
- 二进制：`target/release/instant-space-app`

## 已完成的关键修复

### 1) HAPI 会话 ctx 剩余 0%（已在 hapi-src 修并部署）

- 文档：`/root/opt/newapi-stack/hapi-session-repair/ctx-remaining-zero-fix.md`
- 原因：
  - UI 用了累计 `total.inputTokens` 对比有限窗口，长会话会永远显示 0%。
  - 自定义超大窗口（如 `1e12`）会污染百分比/元数据。
  - 无模型元数据时 fallback 窗口 258400，累计已超窗口。
- 修复：优先用 `last`；拒绝非物理窗口；StatusBar 清洗 contextWindow。
- 用户需硬刷新 HAPI 页面（Ctrl+Shift+R）。

### 2) 手机端 UI（本仓库，样式版本 ui-v59 → ui-v60）

目标：手机端更合理、更像 App，而不是桌面布局硬挤。

改动文件：

- `app/src/app.rs`：样式缓存 `ui-v59 → ui-v60`
- `app/src/pages/space.rs`：
  - 聊天时间从完整 `OffsetDateTime` 显示改为 `YYYY-MM-DD HH:MM`
  - 空间页 header 拆成 status row + action row，方便手机排版
- `app/src/pages/admin.rs`：admin 仪表盘语义化结构 + 审计表横向滚动
- `app/style/ui-system.css`：
  - 全局 box-sizing / 手机密度
  - 空间聊天：消息区占主视觉；composer sticky 在 bottom-nav 上方；底部四 Tab 导航
  - 首页 hero：缩小超大标题、减少视觉高度浪费、按钮更易点
  - 消息气泡左右分栏（even 右对齐紫色浅底）
  - 桌面 ≥761/980：聊天左、攻略/分享右

## 约束（不要踩坑）

- 不要重启无关服务（如 cliproxyapi）
- 不要破坏 map/WASM/chat websocket
- 不要把 `.env` / 密钥 commit
- 用户偏好中文、直接出结果；**未明确要求不要 commit**
- 线上路径前缀是 `/inspace`

## 构建 / 部署

```bash
# 检查
cargo check -p instant-space-app

# 发布（需要 WASM 时再跑 npm）
cargo build -p instant-space-app --release
# 如改了 hydrate/WASM UI：
# npm run build:wasm
systemctl restart instant-space-rust
```

样式改了只 bump `app.rs` 里 `?v=...-ui-vXX` 即可让浏览器拉新 CSS。

## 目录文档

- `docs/ARCHITECTURE_RUST.md`
- `docs/PRODUCT_VISION.md`
- `docs/PHASES.md`
- `docs/NGINX_WEBSOCKET.md`
- `docs/superpowers/`
- 本文件：`memory.md`（模型记忆）

## 待续注意

- 用户曾说手机聊天“完全没改好”；优先保证：时间可读、输入框不被底栏挡住、header 不占半屏、消息可滚。
- 未提交 diff 时先看 `git status` / `git diff`，不要盲 push。

### 3) 手机密度压缩 v60

- 首屏去掉嵌套卡片感：目的地区块去边框背景；手机隐藏 hero visual
- 空间页 header 变薄条；描述隐藏
- 消息气泡去厚边框/阴影；消息列表更扁
- guides/share 卡片 padding 缩小、去重阴影
- 仅 CSS + cache bump；先不强制全量 release（省资源）

## ui-v61 UX Pro Max + SEO foundation (2026-07-21)

### UX (ui-ux-pro-max skill)
- Touch targets ≥44px on key controls
- Safe-area for topbar / bottom-nav / composer / modals
- Focus-visible rings; reduced-motion respect
- Phone: flatten home destination card nesting; hide hero visual
- Space room denser header; flatter message bubbles
- inputs font-size 16px to prevent iOS zoom

### SEO / Search GEO foundation
- Meta description, OG/Twitter, canonical, robots meta in `app.rs`
- JSON-LD WebSite / Organization / WebApplication
- `/inspace/robots.txt` + `/inspace/sitemap.xml` (public spaces + published guides)
- Submit sitemap in Google Search Console: `https://opctoai.com/inspace/sitemap.xml`

## 2026-07-21 — 全站 UI/UX Pro Max v62
- 用户要求“所有页面都用 skill 优化，包括网页端”。按 UI/UX Pro Max 的无障碍、触控、安全区、响应式、表单与反馈规范完成全站统一 v62 层。
- `app/style/ui-system.css`：为首页地图之外的所有内容页统一最大内容宽度/桌面阅读行长、卡片和表单节奏、44px 触控目标、禁用与成功/错误状态、指南/空间/后台布局；移动端统一 14px gutter、单列表单、可换行动作区、后台导航横向滚动与表格滑动提示；包含高对比与 reduced-motion 支持。
- `app/src/app.rs`：加入键盘可见的“跳至主要内容”链接，CSS cache 更新为 `ui-v62`。
- 各路由的 `<main>` 均增加 `id="main-content"`：首页、聊天室、登录、我的空间、指南列表/详情/编辑器及全部 5 个后台页面；404 fallback 也覆盖。
- 验证：`cargo check -p instant-space-app` 通过；release 构建通过（4m22s）；本地真实服务重启后，10 条公开/登录/空间/后台路由在 1440px 与 390px 视口均 200、无页面横向溢出、每页有 main/skip link、加载 `ui-v62`。
- 部署：仅重启 `instant-space-rust`，未触碰其他服务；本地和 `https://opctoai.com/inspace` 均返回 200。

## 2026-07-25 — 关键部署根因 + 首页重构启动 (inspace-v2)

### ⚠️ 根因：为什么"刷新也没用还是旧版"
- systemd 服务跑的是 `/usr/local/bin/instant-space-app`（WorkingDirectory=/root/opt/instant-space-rust）
- 但 `scripts/deploy.sh` 只 `cargo build` 到 `target/release/`，**从不复制到 /usr/local/bin**
- 所以 `systemctl restart` 一直重启旧二进制 → 新代码/新 CSS 版本永不生效
- **正确部署**：build 后必须 `cp target/release/instant-space-app /usr/local/bin/instant-space-app` 再 restart
- CSS 版本号 `?v=` 编译进 `app.rs`，浏览器按 URL 缓存；改 CSS 必须 bump 版本 + 重新 build + 复制二进制才会到浏览器
- CSS/JS 本体由 ServeDir 从磁盘 `app/style` 实时读（改内容无需 build），但版本号变更需要 build

### 首页文案来源（重要）
- 线上首页文案来自 DB 表 `site_page_configs.published_config`（page_key='home'），**不是** Rust 默认值
- 改文案：直接 UPDATE DB 的 published_config + draft_config 即可，**零重启零编译**
- 已确认 A 款标题写入 DB：hero.title=走到导航的尽头，才是体验的开始。/ hero.body=副标题 / hero.note=slogan(Be IN the space, beyond the map.)

### 本轮已完成 (v2)
- 品牌 InSpaceOS → inspace（header.rs + DB seo）
- 手机首页 hero 标题从 clamp(3rem,14vw,4rem) 4行 → clamp(2rem,8vw,2.55rem) 2行
- 手机 hero 预览卡 min-height 540→360，缩小
- 对比度：--inspace-muted #526b7d→#41576a，--inspace-subtle→#4b6070，note色#71889a→#43566a
- CSS 版本 → 20260725-inspace-v2；已 cp 二进制到 /usr/local/bin 并重启，线上已生效

### 待续（本次大重构剩余）
- 底部导航 4→5 Tab（发现/攻略/分享/讨论/我的）
- 探索/攻略：先分类金刚区 + 分页（后端 list_spaces/list_guides 加 category/limit/offset+总数）
- 空间页手机重排 + 讨论独立成页 /inspace/spaces/:id/chat + 修聊天抢滚动

## 2026-07-25 — inspace-v2 大重构「需求基线」（用户确认版，全面开工）

> 用户已明确"全面开工"。以下是逐轮确认锁定的完整需求，后续任何会话以此为准，不要再问已定项。

### 全局基调
- 布局：ChatGPT 式**左侧固定导航 + 右侧内容**（当前是顶部横向导航，要改成左右结构）
- 品牌名统一对外显示 **inspace**（代码里 Instant Space / SpaceOS / InSpaceOS 混用 → 统一 inspace）
- 首页主叙事走**愿景高度**（白皮书/寄语的"介观层/体验的开始"），承接内容落到 Phase 1 真实能力（空间/攻略/地图/扫码），避免吹大点进去没东西
- 对比度：正文实色 `#0f172a`(主)/`#334155`(次)，最弱不低于 `#47569`，全部 ≥4.5:1；废弃半透明灰字与透明叠层文字

### 首页文案（已定，来源=白皮书/寄语，非自编）
- 主标题(A)：**走到导航的尽头，才是体验的开始。**
- 副标题：**地图带你到达，inspace 让你真正进入——看它的过去、此刻在场的人、以及属于这里的故事。**
- 尾签 slogan：**Be IN the space, beyond the map.**
- 注意：线上首页文案来自 DB `site_page_configs.published_config`(page_key='home')，改文案 UPDATE DB 即可，零编译零重启
- 第一屏中文正文 ≤3 行；删掉旧的 4 个基线 chip + 轮播/漂浮元素
- 视线动线：标题 → 搜索框(主 CTA,高亮主色) → 精选空间卡片 → 地图入口；只有搜索是主行动，其余降级
- hero 用深色块(近黑 #0f172a)承托白字标题，与下方浅色内容区分层
- 精选/地图卡片加视觉引导（点位脉冲、"进入空间"箭头）

### 左侧导航（金刚区 + Tab 差异化）
- 桌面 240px 固定侧栏，可折叠 64px 图标栏
- 项目：发现地图 · 探索空间 · 攻略 · 我的空间 · 创建空间 ── 语言/账号
- 每项配专属图标 + 一句微文案（强化金刚区业务属性）
- 当前项主色实心 + 图标微动效；非当前项中性色
- <1024px：侧栏收起为抽屉（顶部汉堡打开）；手机主导航走底部 5 Tab

### 探索 & 攻略（先分类，后分页）
- 进页先给**分类金刚区**：探索=景点/美食/城市街区/活动；攻略=国家→城市（沿用现有地区数据）
- 选分类后 → 网格 + **页码分页**，每页 24 条，显示总数；顶部保留搜索，可跨分类搜
- 卡片网格响应式：手机 1 列 / 平板 2 列 / 桌面 3–4 列
- 后端 `list_spaces` / `list_guides` 增加 `category` / `limit` / `offset` + 返回总数
- 1 万条也只看某分类某页 24 条，不靠无限下滑

### 空间页（手机重点修 + 讨论独立成页）
- 从上到下：①空间信息大卡（做详细：封面/名称/公开·私密/地点/在线人数/主理人 + 主按钮：分享·二维码、写攻略、进入讨论）②攻略区 ③分享/二维码 + 社群入口
- 讨论**不内嵌**，点"进入讨论"跳独立页 `/inspace/spaces/:id/chat`
- 手机滚动修复：页面整体正常竖滚，聊天区不再抢占页面滚动（去掉吃手势的 overflow）
- 底部 5 Tab：**发现 / 攻略 / 分享 / 讨论 / 我的**，固定 + 安全区适配 + 44px 触控，底栏高度纳入内容 padding，修正错位

### 要落地的 UX 技巧（用户点名）
- 优化视觉引导，确立核心转化路径 / 优化交互操作路径
- 增强 tab 栏的差异化和趣味性
- 强化金刚区的业务属性
- 重色/黑元素植入，强化空间层次
- 地图类卡片添加视觉引导

### 执行顺序
1. 左右布局骨架 + 底部 5Tab + 安全区/对比度基线
2. 首页（A 标题 + 愿景，精简 + 视觉引导 + 深色层次）
3. 探索/攻略：分类金刚区 + 分页（含后端 limit/offset）
4. 空间页重排 + 讨论独立成页 + 手机滚动修复
5. cargo check → 正式部署 → 桌面/手机截图验收

### 部署纪律（血泪坑，务必遵守）
- 改 Rust：`cargo build -p instant-space-app --release` → **`cp target/release/instant-space-app /usr/local/bin/instant-space-app`** → `systemctl restart instant-space-rust` → 健康检查
- 改 hydrate/WASM UI：`npm run build:wasm`
- 改 CSS 版本号：bump `app.rs` 里 `?v=...`（当前 `20260725-inspace-v2`）
- 验收：`shot.mjs` 截桌面(1440)+手机(390)图

## v5/v6 轮（2026-07-26）

### Skill 体系
- 总控 skill：`/root/.codex/skills/inspace-design-engineer/`（编排 impeccable + web-design-guidelines 等）
- 主设计 skill：`impeccable`（v4.0.2）；UI 修改后必须跑 `node /root/.codex/skills/impeccable/scripts/detect.mjs --json <改动的css/组件>`
- 仓库根有 `PRODUCT.md`（impeccable init 规范：定位=介观层，标题、Slogan、5 条原则）

### v5 完成
- workspace.css "v5 refinements"：hero 标题压缩、journey 数字弱化、guide-browser 拍平、登录页去横幅、空间页底部快捷导航浅色化
- backoffice.css：admin 手机 390px 横向溢出修复（根因：main.css:1552 stats-grid 4 列 + audit table min-width:760px 撑开隐式网格；修法：`.admin-layout` 等 `grid-template-columns: minmax(0,1fr) !important` + `overflow-x: clip`）——**已复验全部 admin 路由 390px 溢出=0**
- app-shell.css：折叠态 login-link 隐藏、移动断点 1099px、explore chip 横滑

### v6 完成
- impeccable 检测器修复 2 个 warning：侧栏 `transition: width` → 只 transform；`.app-main` 去掉 margin-left 过渡（layout thrash）。剩 1 个 advisory（hero 装饰网格背景，属地图产品语境，保留）
- 修复空间页嵌入聊天消息被压扁（flex 滚动容器里 item 被 shrink 到 31px 互相重叠）：workspace.css 末尾加 `.chat-message { flex-shrink: 0 !important }`
- CSS 版本 bump `20260726-inspace-v6`（需重建二进制）

### QA 结论（v6 前置验证）
- 14 路由 × 1440/390：横向溢出全 0、console 错误全 0
- 空间页 390 整页滚动正常（wheel 600 → scrollY 600）
- QA 登录方式：psql 直插 sessions（token 明文即 token_hash 列，无哈希），playwright addCookies `instant_session`
- 数据库：`postgres://instant_space:***@127.0.0.1:5432/instant_space_rust`（密码见 systemd unit）；spaces 表可见性字段是 `is_public`（不是 visibility）

### 已知遗留
- login server fn 偶发 500（sessions INSERT 慢 >1s），未修
- 首页 hero 底部旧装饰模块（GUIDE/THE BUND 卡、蓝 CTA 横条）保留中，用户未表态

## v7 — 视觉世界重构「测绘图记 / Field Survey」（2026-07-26）

用户诉求：不只是版面，是**审美 + 文案 + 高级感**整体重做。按 impeccable 的 redesign 流程（替换视觉世界，非打补丁）。

### 视觉世界
把「SaaS 深蓝 + 卡片堆 + 渐变 hero」整体替换为**测绘图记**：产品本身就是"地图之后的那一层"，所以界面做成一本实地勘测笔记。
- 纸底 `#faf8f4` / 墨 `#171512` / 朱红 `#b23a29`（唯一强调色）/ 发丝线 `#ddd6c7`
- 圆角 2–3px（原 12–18px），去阴影去渐变，靠**横线 + 留白 + 字重**分层
- 卡片只保留一处：首页 hero 的"空间记录示例"卡（唯一独立对象）

### 字体（高级感的最大单点）
- 自托管 **Noto Serif SC 600**（中文标题）+ **Noto Sans SC 400/500**（正文）
- 316 个 unicode-range 分片 woff2 存 `app/vendor/fonts/`（4.5MB 总量，浏览器只取用到的几片）
- `app/style/fonts.css` 由脚本从 fontsource 的 index.css 生成 288 个 @font-face
- `/vendor` 已有 ServeDir，**加字体不需要改 Rust**

### 新文件
- `app/style/inspace-world.css`（唯一主视觉层，最后加载）——含：token 重定义、**接管全部遗留调色板变量**（`--inspace-*` / `--ui-*` / `--ux-*` / `--color-*` / `--home-*`），这是让整站换色的关键；否则 `body .button-primary` 这类高特异性遗留规则会盖回蓝色
- `app/style/fonts.css`

### 首页结构（app/src/pages/home.rs 改写）
- hero：标题 + 一段 lede + 两个按钮 + slogan；右侧"空间记录示例"卡（标注「示例」，不伪造真实数据）
- journey：去掉 01/02/03 序号，改三栏 `<dl>` 竖线分隔
- guide：假的 GUIDE 深色卡 → 真实 `<table>` 日志（路线/避坑/现场），带 caption 标注示意
- host：横幅 CTA → 结尾 colophon 版式
- 类名：`survey-hero` / `survey-sheet` / `survey-passage` / `survey-stages` / `survey-plate` / `survey-log` / `survey-colophon` / `survey-kicker`

### 文案（crates/domain/src/site.rs 默认值 + DB published_config 同步）
去掉"全球介观空间网络"这类内部术语，改成用户视角的具体话：
- eyebrow「到达之后」/ 主标题保留「走到导航的尽头，才是体验的开始。」
- body「地图把你送到门口。这个地方怎么走、什么时候来、哪里会踩坑，写在这里；还没写下的，问此刻正在现场的人。」
- journey 三段：到达 / 看懂 / 问人
- demo 数据改为具体可信内容（"南京东路站 2 号口出，沿滇池路步行 8 分钟"），全部标注示例
- **改 DB 文案的 SQL 模式**：`UPDATE site_page_configs SET published_config = published_config || jsonb_build_object('hero', published_config->'hero' || '{...}'::jsonb)`

### 关键经验
- 遗留 CSS（ui-system.css 7135 行 / main.css 6399 行）大量 `body .x` 高特异性规则。**不要逐条打补丁**，直接在 world 层接管它们的 CSS 变量，一次性换掉全站配色。
- 少数顽固规则（button-primary/language-button）需要用 `body .x` 同级特异性覆盖。
- fullPage 截图会把 fixed 底栏和滚动前的旧渲染合成进来，**看到"斑马纹紫底"先用 getComputedStyle 验证**，不要照着截图瞎改。

### QA
- 5 路由 × 1440/390：横向溢出全 0、console 错误全 0
- impeccable detect.mjs：0 findings
- CSS 版本 `20260727-survey-v2`

## v8 — 地图恢复、独立聊天页与工作台全栈修复（2026-07-26）

### 地图与资源
- 修复生产前缀：`app/src/map_boot.js` 通过 `assetBase()` 在 `/inspace` 下加载 `/inspace/vendor/maplibre-gl/*`，地图模式恢复。
- `crates/map-ui/src/maplibre_shim.js` 的投影/样式切换改到 MapLibre `idle` 安全边界，并捕获过早 `setProjection`；线上已无 `Style is not done loading`。
- 字体旧 unicode shard 中有 173 个 0 字节文件；已删除 316 个分片，改成 3 个完整自托管 WOFF2（Noto Sans SC 400/500、Noto Serif SC 600），总量 3.7MB。
- nginx 的 WASM 缓存版本更新到 `20260726-ui-v66`；CSS 生产缓存使用 `20260728-survey-v5`。配置备份：`/etc/nginx/conf.d/opctoai.com.conf.bak-inspace-v8-20260726`。

### 空间与聊天 IA
- 空间详情页不再嵌入聊天；聊天独立路由：`/inspace/spaces/:space_id/chat`（同时保留无前缀路由）。
- 独立聊天页保留访问控制、历史消息、WebSocket 在线状态与发言框；手机端从消息区域滑动会滚动页面，不再锁住手势。
- 私密聊天路由在访问码验证前不渲染任何消息；QA 测试为 0 个消息节点。

### 用户工作台与攻略入口
- 工作台增加“先建空间 → 在空间里写攻略 → 发布后分享二维码”三步说明。
- 每个空间卡新增：打开空间 / 写攻略 / 讨论区 / 管理空间；写攻略链接始终携带 `space_id`。
- 空间详情、管理弹窗、攻略浏览页均增加空间内写攻略入口；编辑器通过 `?space_id=` 自动选中空间并预填省市地点。
- 管理弹窗操作按状态/危险操作分组，反馈中文化；保存、暂停、重新开放已在生产真实点击验证。关键修复：操作成功后不立即刷新外层列表，避免弹窗被卸载；关闭弹窗时才刷新。删除仍立即刷新。
- 修复管理员空间分页按钮中 Rust 表达式被渲染到页面的问题；静态扫描无 `href="#"`、空点击处理器或 `todo!()`。

### 后端与数据库
- 新增 session 过期索引迁移并清理历史过期记录：`sessions_expires_at_idx`、`access_sessions_expires_at_idx` 均已存在，过期 session 为 0。
- 生产旧表原所有者为 `postgres`，首次迁移因应用账号无 DDL 所有权而启动失败；已将 `sessions` / `access_sessions` 所有者调整为 `instant_space`，迁移成功，服务恢复。
- `create_session()` 增加过期 session 清理，修复历史登录偶发慢 INSERT/500 风险。

### 生产 QA
- 地图：MapLibre object、1180×900 canvas、瓦片可见、0 page error、0 failed request。
- 独立聊天：WebSocket `connected`、历史消息与输入框存在；私密消息无泄漏。
- 390px：空间详情/聊天/工作台横向溢出均为 0；核心按钮均至少 44px；管理弹窗无横向溢出。
- 管理按钮：保存成功；暂停后按钮状态切换；重新开放后恢复测试空间为 `active`；0 console/page error。
- 最终证据保存在 `output/playwright/`：地图、空间页、聊天页、工作台、管理弹窗、管理员空间页截图及 `qa-results.json` / `qa-final.json`。

## v9 — 修复「地图上不显示空间标记」（2026-07-26）

**症状**：创建空间时在地图上标点，`/inspace/map` 上看不到任何标记。

**根因**：v8 IA 重构时新建了 `app/src/components/map_workspace.rs` 作为 `/map` 与 `/inspace/map` 的唯一地图组件，但它只调用 `instant_map_ui::mount()`，**从未加载空间数据、也从未调用 `sync_points()`**。原来负责渲染标记的 `MapHome`（`app/src/components/map_home.rs`）在重构后已不被任何路由引用，成为死代码，标记逻辑随之失效。

**次因**：`MapMarkerSync` 的隐藏代理按钮用的是 `data-space-open`，而 `crates/map-ui/src/maplibre_shim.js` 的 marker click 查询的是 `[data-space-select="{id}"]`，属性名不一致 → 即使有标记，点击也打不开详情。

**修复**：
- `app/src/components/map_home.rs`：`MapMarkerSync` / `SpaceDetailDrawer` / `SpaceListSkeleton` 改为 `pub`；代理按钮属性 `data-space-open` → `data-space-select`（与 shim 对齐）
- `app/src/components/map_workspace.rs`：新增 `Resource` 调用 `list_spaces()`（跟随 `refresh.spaces` / dest 筛选信号），挂 `<MapMarkerSync>` 和 `<SpaceDetailDrawer>`，标题下加 `.map-workspace-count` 显示「N 个空间在地图上」
- `app/style/inspace-world.css`：`.map-workspace-bar` 标题块加纸底卡片 + 墨色字（原来白字压在浅色瓦片上不可读）
- `.gitignore`：新增 `/output/`（QA 截图产物 3.6MB 不入库）

**验证**（真实浏览器，线上 https://opctoai.com/inspace/map）：`markerCount=3`、`libreMarkers=3`、`proxyCount=3`、`drawerOpened=true`、`consoleErrors=[]`。脚本 `tests/browser/map-marker-check.mjs`。

**注意**：本地 127.0.0.1:3001 直接访问 `/inspace/map` 会因 `assetBase()` 走 `/inspace` 前缀而 404 加载不到 maplibre，**地图标记验证必须走线上域名**。

**版本号**：二进制 CSS 版本已 bump 到 `20260728-map-v6`；因随后有纯 CSS 改动，nginx sub_filter 追加 `'20260728-map-v6' -> '20260728-map-v7'`。WASM sub_filter 版本同步为 `20260728-map-v6`。nginx 备份 `/etc/nginx/conf.d/opctoai.com.conf.bak-map-v6`。

## v10 — 首页质感 / iPad 横屏遮挡 / 攻略删除 / 聊天页重做（2026-07-26）

### 1. 攻略删不掉（真 bug）
**根因**：`delete_guide` 服务端实际调用的是 `archive_guide`（仅 `status='archived'`），而管理列表 `list_guides_by_space(include_unpublished=true)` 会把 archived 一并列出 → 用户点删除后条目仍在，只是变「已归档」。`guides` 表无外键被引用，可以物理删除。

**修复**：
- `crates/db/src/guides.rs`：移除 `archive_guide`，新增 `delete_guide_row()`（`DELETE FROM guides ... RETURNING`）
- `app/src/server/guides.rs`：`delete_guide` 改调 `delete_guide_row`
- `app/src/pages/host.rs` `RelatedSpaceGuides`：`archive_action` → `delete_action`，新增 `pending_delete` 两步确认（首次点击变「确认删除」，再点才执行），成功提示「攻略已删除」
- `app/style/inspace-world.css`：`body .related-guide-item` 改 grid 布局（标题+状态同行、操作靠右），原来三段垂直堆叠占满屏

**验证**：`tests/browser/delete-guide-test.mjs` 真实浏览器点击 → DB 行数 3 → 2，无 console error。

### 2. iPad 横屏文字被遮挡
**根因**：1180×820 下右上角「登录」被压成竖排两行；`.shell-topbar-login` 缺 `white-space:nowrap` 与 `flex:0 0 auto`。另有关闭态抽屉侧栏留在画布外导致 390px 文档横向溢出（sw=394）。

**修复**（`app/style/app-shell.css`）：topbar 各元素加 `flex:0 0 auto` + `nowrap`；`.shell-topbar-context` 限宽 210px + ellipsis；`.shell-global-search` 加 `min-width:0`；新增 `@media (min-width:1100px) and (max-width:1279px)` 隐藏 `.shell-topbar-path`；`html:has(.shell-sidebar:not(.is-open)), body { overflow-x: clip }`。

**验证**：1024 / 1112 / 1180 / 1194 / 1366 横屏全部 `sw == vw`、遮挡元素 0。

### 3. 聊天页重做
- `app/src/pages/space.rs`：新增 `split_chat_stamp()` / `sender_monogram()`；header 改「返回箭头 + 空间名 + 单行 caption（状态点 + 在场人数）」；消息改两列（首字母头像 + 正文块）；空态换 `.chat-empty`；发送按钮加纸飞机 SVG 且输入为空时 disabled；textarea `rows=1` 自动增高。
- `app/src/chat_realtime.js`：`appendMessage` 适配新 DOM；`isPinnedToBottom()` 仅在贴底时自动滚动（往上翻历史不再被拽回）；Enter 发送 / Shift+Enter 换行；状态文案精简；时间统一 24 小时制（`getHours/getMinutes` 补零，替代 `toLocaleTimeString` 的 `07:09 AM`）。
- `app/style/inspace-world.css`：`.chat-room` 重写为三行 grid（header / 滚动区 / 吸底 composer），`width: min(100%, 780px)`，`height: calc(100dvh - topbar - 56px)`。

**踩坑（重要）**：`ui-system.css` 有多处高特异性规则打架 —— 给 `.chat-message-list` 写死 `height: min(52dvh,620px)`、给 shell 设 `align-items:start` 与命名 grid-area、900px 以下把 shell 切回 flex、把状态文案强制成 34px 药丸。全部在 `inspace-world.css` 末尾用 `body .xxx { ... !important }` 定点压制并注明原因。

**验证**：`chat-send-test.mjs` → 消息 10→11、status connected、errors []；390px `sw==vw==390`。

### 4. 首页质感
- hero 标题 `clamp(2.75rem,5.1vw,4.5rem)` + display 衬线 + `letter-spacing:-.03em` + `max-width:13ch`；hero 改 `align-items:center`、`min-height:min(72vh,620px)`；移除勒死正文的 `.survey-hero-copy{max-width:20ch}`。
- slogan `.survey-note` 改 display 斜体 + 墨色实线上边框（签名感）；`.survey-sheet` 去掉 1px 幽灵边框，改分层阴影。
- 唯一一处编排动效：`survey-settle`（sheet 落纸 620ms）+ `survey-rule-draw`（三条横线依次画出，用 `::before` 画线避免文字被 scaleX 压扁），含 `prefers-reduced-motion` 兜底。
- 清理三处 v7 时期互相打架的旧 patch 块，hero cascade 现在是干净的。
- 修 `main.css` 深色主题遗留：`.space-detail-guide-list a` / `.related-guide-main .guide-list-link` 白字深块 → 纸底可读的下划线列表项。

**版本号**：二进制 CSS 版本 `20260729-craft-v1` → `20260729-craft-v12`；nginx 里针对 v1 的 `sub_filter '20260729-craft-v1' '20260729-craft-v11';` 已随之移除（两处 location）。

## v10 — 首页：随滚动被"绘制"的测绘图记（2026-07-26）

**问题**：首页正确但静止 —— 所有内容一次到位，600ms 之后不再有任何编排。用户明确否掉了上一版"深色墨版 hero"（纯静态换色），要求「得奖作品级 + 可动效果」。

**方案**：不换配色、不加装饰，改为让整页**随滚动被绘制出来**（scroll-driven，纯 CSS，无 JS、无定时器）。

改动全部在 `app/style/inspace-world.css` 末尾（`v10 — HOME: the sheet is drawn while you read it`）：

1. **左侧页边基准线**：`.inspace-home-modules::before`，`animation-timeline: scroll(root block)`，`scaleY` 随文档进度推进；顶部 34% 朱红、其后发丝灰。<1100px 隐藏。
2. **开场编排**：h1 用 `clip-path` 竖向擦除（`inset(-12% -20% 104% -6%)` → `inset(-12% -20% -14% -6%)`，横向留负值，绝不切字），lede / actions / note 依次 `home-write` 入场，最后在 lede 上方划出 92px 朱红基准线。
3. **hero 两平面分离**：`.survey-hero-copy` 随滚动 recede（opacity→.12，Y→-46px，range `0 62vh`）；`.survey-sheet` 反向 hold（Y 26px→-34px，微旋 .5deg→-.35deg，range `0 88vh`），双 timeline 写法：`animation-timeline: auto, scroll(root block)`。
4. **每段"先划墨线，再写行"**：`.survey-stages::before` / `.survey-log::before` 用 `animation-timeline: view()` + `animation-range: entry 22% entry 74%` 划线；三个 stage / 三行 log 各自错开 5% range 写入。
5. **论点被证据挟持**：≥1100px 时 `.survey-plate-copy` `position: sticky`（top = topbar + 56px），左栏文字停住，右侧 log 表滚过。
6. **按钮**：secondary / colophon 按钮改成墨色从左侧填充（`::before` scaleX + `isolation`），不再只是变色。

**排版修正（CJK 关键）**：
- `max-width: 12ch` 让 h1 被截断 —— `ch` 按 "0" 字宽计，汉字是它的两倍。display 改 `max-width: none` + `clamp(2.6rem, 4.5vw, 4.1rem)`；lede 改 `max-width: 20em`。
- 新增 `@media (min-width:1100px) and (max-width:1339px)` 重述 iPad 横屏 hero 尺寸（原来那条 band 规则因源顺序已失效）。

**兜底**：`@supports not (animation-timeline: view())` → 所有线 scaleX/scaleY = 1，页面即"已完成的图记"；`prefers-reduced-motion: reduce` 全部 `animation:none` + opacity/transform/clip-path 复位（已实测）。<1099px 关掉 hero 视差（手机上 hero 就是整屏，recede 会在拇指下变空白）。

**验证**（全部走线上 https://opctoai.com/inspace）：
- 1440 / 1180 / 390 三档 `sw == vw`，console errors = 0
- `CSS.supports('animation-timeline','view()')` = true；滚到底实测 rule scaleY=1、heroCopy opacity=.12 translateY(-46)、sheet translateY(-34) 微旋、stages rule scaleX=1
- 八个滚动位置逐一采样：passage/stages/plate/log/colophon 的 opacity 最终都达 1（不存在"滚过去还是空的"）
- reducedMotion=reduce：h1/log 行/按钮 transform=none opacity=1，两条线 scale=1
- 回归：地图标记 3/3 + 抽屉可开（`BASE_URL=https://opctoai.com`）、聊天 11 条消息 connected、iPad 1024/1112/1180/1194/1366 全部 `sw==vw` 且 covered=[]
- `impeccable/scripts/detect.mjs` → `[]`

**发布**：纯 CSS，未重建二进制。nginx `sub_filter '20260729-craft-v12' '20260729-craft-v15';`（两处 location）+ reload。二进制里仍写 v12，nginx 负责改写到 v15；下次纯 CSS 改动继续 bump 这条 sub_filter 的右值。

**注意**：`map-marker-check.mjs` 的 `BASE_URL` 默认值已改为 `https://opctoai.com`（本地 3001 因 assetBase 前缀会 404，误报 markerCount=0）。

## v11 — 首页：实拍图片带 + JS 视差；`keep-all` 遮挡 bug（2026-07-26）

### 1. 「文字被挡住」的真因：`word-break: keep-all`
用户 iPad 截图里 `.survey-plate-copy h2` / `.survey-colophon h2` 压在右侧日志栏底下。实测（1194/1366/1440）文字比自身盒子宽出 133–219px。

根因在 `inspace-world.css` 原第 1148-1151 行：给四个标题加了 `word-break: keep-all`，本意是防英文单词断行——但**中文没有空格，整句被当成一个不可断的"词"**，永远不换行，直接溢出。

改为 `word-break: normal; overflow-wrap: normal; line-break: strict;`。同时把这一带所有 CJK 度量从 `ch` 换成 `em`（`ch` 按 "0" 字宽算，汉字是两倍）：
- `.survey-passage-head` 44ch→34em、`h2` 16ch→15em（≥1100px 那条 22ch→16em）、`p` 42ch→30em
- `.survey-plate-copy h2` 13ch→12em
- `.survey-colophon h2` 14ch→13em

实测溢出全部转负（-34 ~ -251px）。

### 2. 图片：六张 CC0/PDM 实拍，自托管
- 来源：Openverse API（`license=cc0,pdm`），逐张目视筛选后下载
- 处理：PIL 居中裁 3:2 → WebP q74，两档宽度 720/1080，共 12 个文件 872KB
- 位置：`app/vendor/img/`，授权清单 `app/vendor/img/CREDITS.md`（记录标题/许可/作者/原始链接，CC0 不强制署名，但保留可审计链路）
- 选图：外滩、里斯本电车坡道、首尔北村巷、威尼斯水巷、港湾夜灯、山口观景点

### 3. 新 section `.survey-field`（order 25，插在 passage 与 plate 之间）
`app/src/pages/home.rs` 新增 `FieldPlate` 组件（`slug` / `zh` / `en` / `depth`）。要点：
- `srcset` 两档 + `sizes="(max-width:720px) 78vw, (max-width:1099px) 42vw, 27vw"`
- `alt=""`（装饰性，语义由 figcaption 承担）、`loading="lazy"` `decoding="async"`
- `ul > li` 真列表语义
- **不是卡片**：无边框、无圆角堆叠、无阴影层；只有 `box-shadow: 0 1px 0 var(--rule-strong)` 一条接触边 + `filter: saturate(.86)` 压进纸色，hover 才回饱和
- 三列错落（2/3/5/6 各有不同 `margin-top`），错落是视差能被看见的前提

### 4. JS 视差 `app/src/field_parallax.js`（新增，`include_str!` 进二进制）
为什么要 JS 而不是继续用 scroll timeline：每块图深度不同，且**手指离开后要继续缓动一拍**（scroll timeline 精确贴合滚动位置，观感机械）。
- 契约：`[data-parallax-strip]` + 每块 `[data-depth]`
- 进度 = `(vh - box.top) / (vh + box.height)` 映射到 ±78px，再乘各自 depth
- 缓动 `y += (target-y)*0.085`，差值 <0.05 时停 rAF（不空转）
- `IntersectionObserver` 只在可见时算；`prefers-reduced-motion` 监听 change 事件，切到 reduce 立即 `clear()`
- WASM hydration 会重建 DOM，所以 `popstate` + 捕获阶段 click 之后延时 re-attach

### 5. nginx：srcset 不会被自动加前缀
原来只重写 `src="/vendor/` 和 `href="/vendor/`。新增两条（两处 location 各一份）：
```
sub_filter 'srcset="/vendor/' 'srcset="/inspace/vendor/';
sub_filter 'w, /vendor/'      'w, /inspace/vendor/';
```
第二条处理 srcset 里第二个候选（`... 720w, /vendor/...`）。

### 6. 踩坑：`height="480"` 属性锁死高度
`<img width="720" height="480">` 是为了预留盒子防抖动，但没写 `height: auto` 时该属性胜过 `aspect-ratio`，图被固定成 480px 高（实测 343×480，严重变形）。加 `height: auto` 后恢复 343×228。

### 7. 发布
Rust + 新 JS → 完整流程：`cargo check` → release build（7m09s）→ 原子替换 `/usr/local/bin/instant-space-app` → `build-wasm.mjs`（2m08s）→ restart service。版本号 `app.rs` bump 到 **20260729-craft-v17**，nginx 两处 sub_filter 同步改成 v17，并**删掉了旧的 v12→v16 那条链**。后续纯 CSS 改动：加 `sub_filter '20260729-craft-v17' '20260729-craft-v18';`。

### 8. 验证（全部线上）
- 1440/1194/390：`sw==vw`，console errors 0，`/vendor/img/` 无 4xx
- 视差实测有值且会衰减：1440 下 6 块从 -5.36/-10.05/-3.01/-8.04/-4.02/-9.04 收敛到 ~-0.3
- `currentSrc` 全部命中 720w（列宽 343/322/304，符合 sizes 计算）
- reducedMotion=reduce：6 块 `transform: none`，figure opacity 全 1
- a11y：无 alt 缺失、装饰图 alt 全空、`ul>li` 语义、6 张全 lazy
- 回归：地图标记 3/3 + 抽屉可开、聊天 11 条 connected、iPad 1024/1112/1180/1194/1366 全部 `sw==vw` covered=[]
- `impeccable/detect.mjs` → `[]`

## v12 — 1000 个真实空间 + 1000 篇攻略播种；地图聚类；攻略分页（2026-07-26）

用户要求：10 个著名国家各插 100 个空间、每个空间配一份攻略，中国选 100 个著名景点。

### 1. 采集管线 `scripts/seed/`
| 文件 | 作用 |
|---|---|
| `plan.json` | 10 国 × 城市清单（中英名、坐标、省/州、采样半径） |
| `fetch_places.py` | Wikipedia GeoSearch + Wikidata 采集器 |
| `places.json` | 1000 条采集结果（392KB，**已 gitignore**） |
| `make_seed_sql.py` | 生成空间 + 攻略 SQL |
| `seed_spaces.sql` | 5.2MB 生成物（**已 gitignore**） |

**关键决策：反向查询。** Wikidata SPARQL 端点对"某国全部知名地点"这类查询稳定 504 超时。改为按城市中心用 Wikipedia GeoSearch 取附近条目，再用 `wbgetentities` 批量水化（50 个一批，秒级）。

**六点环形卫星采样**：GeoSearch 半径上限 10km、单次上限 500，大城市覆盖不足 → `geosearch()` 里超过 `MAX_RADIUS` 时改为围绕中心取六个卫星点分别采样再合并去重。

**排序打分**：`pageviews(30d) + sitelinks*400 + heritage?3000`。
**分类**：Wikidata P31 → `TYPE_MAP` 映射到 `space_type`；`REJECT_P31` + `BAD_TITLE_BITS` 过滤人物/城市/战役/列表页。
**手工剔除 5 条**不当条目（中南海、南京人民大会堂、巴黎古监狱、里昂主宫医院、Ulucanlar Prison Museum），从同城补齐。

**确定性 UUID**：`uuid5(NAMESPACE, "inspace:space:{QID}")`，SQL 全部 `ON CONFLICT (id) DO UPDATE`，**重跑安全**。

**攻略正文**：按 `space_type` 选开场白；4 组路线/时段/避坑文案用 `sha256(qid+salt)` 确定性选取；有遗产身份的加"现场规矩"段。**刻意不编造营业时间/票价/电话**。

结果：10 国各 100（China/Japan/France/Italy/UK/US/Spain/Türkiye/Thailand/Egypt），类型 scenic 849 / transit 66 / park 43 / event 39 / food 3；空间 1003、已发布攻略 1001、空间↔攻略配对 1000。

### 2. 标题后缀是索引噪音
初版标题是 `{地点}·实地攻略` / `{name} — a field guide`。1000 行全带同一后缀，目录页变成一列重复文字。已 `UPDATE` 去掉后缀（标题就是地点名），并同步改了 `make_seed_sql.py` 第 190 行，避免重跑再加回来。

### 3. 地图：1003 个 DOM marker → 视口裁剪 + 网格聚类
`crates/map-ui/src/maplibre_shim.js` 的 `renderMarkers` 原本是一个 Space 一个 DOM marker。1003 个在 zoom 3 全糊成黑块，主线程也被拖住。

重构为 `renderMarkers`（收数据）+ `paintMarkers`（画当前视口）两层：
- `visiblePoints()`：按 `map.getBounds()` 裁剪，外扩 15% 便于平移
- `clusterPoints()`：用 `map.project()` 投到屏幕坐标，按 `CLUSTER_CELL_PX = 78` 网格归并；单点仍是原来的 `.map-marker`，多点变 `.map-cluster` 计数气泡
- `MAX_PAINTED_MARKERS = 220` 硬上限兜底
- 气泡点击 `easeTo(zoom + 2.2)` 下钻
- `moveend` / `zoomend` 重绘，监听器存在 `store.onMoveRepaint`，`cleanupStore` 里 `off` 掉

**同时修了一个隐藏 bug**：`fitMapToPoints` 原本每次 sync 都跑，会跟用户的平移缩放打架。现在用 `store.fitSignature`（点 id 拼接）比对，只有筛选/搜索真的变了才重新取景。

样式在 `inspace-world.css` 末尾（纸底 + 墨色描边，hover 转朱红）；`.is-lg` ≥25、`.is-xl` ≥100 三档尺寸。

实测下钻链路：zoom 3 → 11 个气泡 → 5.2 → 7.4 → 9.6 → 11.8 → 14 出现单点 pin，点开抽屉正常。

### 4. 攻略目录：全量 → 分页
`/inspace/guides` 原来一次渲染 1001 条，SSR HTML **691KB**。

- `crates/db/src/guides.rs` 新增 `list_published_guides_page`（`PaginatedGuides { items, total }`），带关键词模糊匹配（标题/地点/城市/省份）
- `app/src/server/guides.rs` 新增 `list_guide_page` server fn + `GuidePageResult`，越界页码回落到最后一页（与 `list_space_page` 行为一致）
- `guide_browser.rs`：加搜索框、`PAGE_SIZE = 24`、`GuideResults` 组件（复用 explore 的 `directory-pagination` 样式）；任一筛选变化时 Effect 把 `page` 复位到 1

SSR 从 691KB 降到 **57KB**。

**行密度**：24 行堆叠三行文字（标题/地区/地点）在桌面上一行 145px，扫读困难。改成单行索引条（标题 | 地区），桌面 76px。地点名已经在标题里，第三列删掉。

### 5. 踩的最大的坑：Cloudflare 缓存了旧的 wasm glue
改完 shim、构建、部署之后，浏览器**依旧**跑旧代码：DOM 里 1003 个 pin、`store` 上没有 `allPoints`。排查过程：
- `curl` 拿 `maplibre_shim.js` → 新代码（有 `paintMarkers`）✅
- `curl` 拿 `instant_space_app_v64.js` → 引用 `?v=8f996055f235` ✅
- 但浏览器实际请求的是 `?v=0455aeedb7e8`（24727 字节的旧文件）❌

原因：`instant_space_app_v64.js` 这个 URL 在 CF 边缘的缓存 `age=30051`（8 小时前），`cache-control: immutable` 一年。**页面里 `?v=craft-vNN` 只作用于 `<script>` 那一次请求，wasm glue 内部 import 的 snippet URL hash 不受它影响**；而 CF 命中的是没有 query 的那份旧 glue。`Network.setCacheDisabled` 也没用，因为问题在边缘不在本地。

解法：`scripts/build-wasm.mjs` 的 `OUTPUT_NAME` 和 `app/src/main.rs` 的 `.output_name()` 一起 **v64 → v65**，换文件名绕开。

**教训：改了 `crates/map-ui/src/*.js`（会被 wasm-bindgen 打成 snippet）必须 bump `OUTPUT_NAME`，只 bump CSS 版本号不够。**

### 6. 发布
版本链：nginx `sub_filter '20260729-craft-v17' '20260729-craft-v20'`（v18/v19 已被覆盖）。`OUTPUT_NAME` = `instant_space_app_v65`。

### 7. 保留脚本
`tests/browser/seed-verify.mjs`（地图聚类 + 攻略分页 + 探索页一次跑完）、`tests/browser/cluster-drill.mjs`（逐级下钻到单点并开抽屉）、`tests/browser/guides-shot.mjs`（桌面/手机行高与溢出）。

---

## v13 — 留痕（Traces）与时间胶囊（Capsules）

用户的原话是：一个空间应该像真实地点一样——旅馆前台有本写满的簿子、黄山栏杆上有锈住的同心锁、墙角有只有两个人看得懂的话。这一版把「地点会留下东西」这件事做成了产品的第二层，讨论区只负责「此刻」，留痕负责「之后」。

### 到场判定（用户拍板：扫码 ➕ 定位 ➕ Discord，三选一）

`app/src/server/traces.rs::judge_presence()`，优先级从强到弱：

1. `scan` — URL 带 `?via=qr`。二维码里编码的就是带 `?via=qr` 的链接（`space_share.rs::scan_url()`），复制链接仍给纯 URL。**这一条是 v13 才接上的，之前二维码编的是纯 URL，扫码到场是死的。**
2. `geo` — 浏览器 Geolocation 拿到坐标，服务端用 haversine 算距离。留痕半径 `TRACE_ON_SITE_RADIUS_M = 800m`（宽松），胶囊用自己的 `radius_m`。
3. `discord` — 勾选「我是这个空间社群成员」**且**该空间确实配了 `discord_group`。
4. `remote` — 都不满足，照样能写，只是标成远程。

距离必须用 haversine，不能用平面近似：种子空间里有一批在 55°N 以上。

### 时间胶囊的规则（用户拍板：A 告诉 B，且 B 必须到场）

- 口令只存 Argon2 哈希（`instant_auth::hash_password`），服务器读不出来。忘了就永远打不开——这是设计，不是缺陷。
- `open_capsule` 判定顺序：已被取走 → 试错锁死 → 未到开启日期 → **到场**（扫码 or 距离 ≤ radius，否则 `PresenceRequired` / `TooFar`）→ 口令。**先判到场再判口令**，所以远程的人连"口令对不对"都探不出来。
- `CAPSULE_MAX_ATTEMPTS = 8`，超了永久锁死。
- `mark_capsule_opened` 带 `WHERE opened_at IS NULL`，并发下只有一个人能开成。
- 一个胶囊只能开一次，开完对所有人显示「已被取走 + 谁 + 何时」。

### 数据

迁移 `crates/db/migrations/20260726000200_traces_and_capsules.sql`：`presence_proof` 枚举 + `space_traces` + `space_capsules`。生产库已执行，`_sqlx_migrations` 已手工补录且校验和验证通过。表属主是 postgres，已 GRANT 给 `instant_space`。

删除用软删（`hidden`），编年史的数字要诚实。

### 前端

- `app/src/components/presence.rs` — `PresenceState`（全 RwSignal）+ `detect_scan()` 读 URL + `request_location()` 走 web-sys Geolocation 回调。web-sys features 要加 `Navigator/Geolocation/Position/Coordinates/PositionError/PositionOptions/Location`。PositionOptions 的 setter 是 `set_enable_high_accuracy` 这种新名字，且不用 `mut`。
- `app/src/components/space_traces.rs` — `SpaceTraces` 挂在空间页攻略列表和讨论入口之间，`id="space-traces"` 供聊天页锚点跳回。
- **空空间的编年史是冷启动钩子**：1000 个种子空间都还是空的，所以空状态不写「暂无数据」，写「还没有人在这里留下任何东西 / 你可以是第一个」，左边一道朱红竖线。
- `CarveButton` — 聊天页每条消息一个凿子图标，把一句话刻进留痕。`proof` 强制 `remote`：刻别人的话不能算你到场。
- 聊天页 header 加「这里留下的」链接 + 一句「讨论会滚走」的说明，房间网格从 3 行变 4 行（`grid-template-rows: auto auto minmax(0,1fr) auto`），`.chat-message` 变 3 列（头像/正文/凿子）。

### 一个必须记住的交互坑

`CapsuleCard` 开信成功后**不能**调 `on_changed` 刷新列表。刷新会重建卡片，把读者刚挣到的信当场抹掉。宁可让编年史的计数暂时不准。第一次 e2e 就是这么发现的：`wrong-pass` 正常，`opened letter: NONE`。

### 首页

`survey-keep` 段（order:26，在 field strip 和 guide plate 之间）。两栏，中间一道细线：左边讲留痕，右边讲胶囊三步（埋下 → 走到 → 说对）。文案从实物切入（写满的簿子、锈住的锁），不从功能切入。

### 又踩了一次 Cloudflare

这一轮**没改** map-ui 的 js，所以按 v12 的教训判断「不用 bump OUTPUT_NAME」——错了。CF 边缘缓存的是 **wasm 二进制** `instant_space_app_v65_bg.wasm`（`immutable`, 1 年），新 glue + 旧 wasm 直接报：

```
WebAssembly.instantiate(): Import #34 "./instant_space_app_v65_bg.js"
"__wbg_navigator_99621db14b3f1099": function import requires a callable
```

诊断方法：`curl` 和 `md5sum` 都说服务端是新的，但 Playwright 里 `fetch(..., {cache:'reload'})` 拿到的响应头是 `cf-cache-status: HIT`。**判缓存要在浏览器里查 cf-cache-status，不要信 curl。**

**修订后的规则：只要 wasm 二进制变了（也就是只要改了任何 Rust 前端代码），就必须 bump `OUTPUT_NAME`。** 不是只有改 js snippet 才要。这一轮 v65 → v66 → v67（v66 是这条教训本身，v67 是修开信那个 bug）。

`OUTPUT_NAME` 在两个地方，必须同步：`scripts/build-wasm.mjs:11` 和 `app/src/main.rs:244`（还有 244 行下面的测试断言）。

另：那个 `setsid bash -c '... && node ...'` 的链式命令有一次整个没跑起来（日志时间戳没变、`ALLDONE` 没写）。改用 `nohup setsid ... </dev/null & disown` 并把 `&&` 换成 `;` + 显式 `echo RC=$?`，才能确认每一步真的执行了。

### 验证

`tests/browser/traces-e2e.mjs`：扫码到场 → 写留痕（校验 proof 标成「扫码到场」）→ 封存胶囊 → 错口令（应答「你站对了地方，但这不是那句话」）→ 对口令（应展开信）→ 手机端零横向溢出 → 零 console 错误。

**注意**：Leptos 受控 textarea 用 Playwright 的 `fill()` 不触发 `on:input`，signal 不更新，提交按钮永远 disabled。必须用 `pressSequentially()`。

---

## v14 — 到场验证改用「现场口令」（Wi-Fi 那套逻辑）

用户的原话：「第一点其实需要和那个打开 Wi-Fi 热点的逻辑一样，在 Wi-Fi 里面看进入空间的密码。」

### 为什么这是对的

v13 的到场判据里，**扫码和定位都是客户端说了算**：`?via=qr` 是谁都能往 URL 后面加的字符串，坐标是浏览器随便报的数字。实测确认过这个洞——伪造 `scanned=true` 从北京发请求，拿到的 proof 就是 `scan`，而且**能直接绕过胶囊的距离判定**。

现场口令不一样：它由服务端拿 Argon2 比对，答案从不下发到浏览器。人得站在屋里，从 Wi-Fi 列表里把它抄下来。

### 复用了已经存在的东西

不用新建一套密码。每个空间本来就有 6 位码，主理人被引导把热点名设成 `InstantSpace_<六位码>`（`crates/domain/src/spaces.rs::hotspot_name()`）。客人打开 Wi-Fi 列表就能看见，人不在现场就看不见。**Wi-Fi 覆盖范围天然约等于物理到场范围**，这比 GPS 靠谱，室内还不掉链子。

- `PresenceProof` 新增 `OnSite`（`onsite`），排在 `Scan` 之后
- 迁移 `20260727000100_onsite_presence.sql`。**PG12 不允许在事务块里 `ALTER TYPE ... ADD VALUE`**，所以文件头必须写 `-- no-transaction`（sqlx 认这个标记，见 `sqlx-core/src/migrate/source.rs:127`）
- 枚举属主是 postgres，`instant_space` 改不了 → 服务启动跑迁移会报 `must be owner of type presence_proof`。**得先用 postgres 手工执行，再把这条补进 `_sqlx_migrations`**（校验和用 sha384 算文件内容）。这一步漏了会导致服务起不来，重启 7 次全挂
- `verify_onsite_code()` 在 `app/src/server/traces.rs`，留痕和开胶囊共用
- `check_onsite_code` server fn 只回 true/false，错了不透露任何信息

### 顺手堵掉的伪造洞（本轮的重点）

判定顺序改成 **口令 → 扫码 → 定位 → Discord**，并且：

- **扫码不能对抗矛盾证据**：如果浏览器同时报了坐标，而坐标在半径外，`scanned` 直接不认。理由是加 `?via=qr` 零成本，而主动交出一个一千公里外的坐标是自己打自己的脸。没给坐标的扫码仍然认（正常扫码用户不一定授权定位）。
- **开胶囊同理**：原来 `if !scanned` 直接跳过整个距离检查，等于 `?via=qr` 能开走全站所有胶囊。现在只有**口令**能无条件跳过距离；扫码仍要接受坐标反证。

`tests/browser/forged-presence.mjs` 是这个洞的回归测试，6 条断言：

```
伪造扫码 + 千里外坐标  -> remote   （堵住了）
伪造扫码 + 不给坐标    -> scan     （仍然宽容）
真在现场扫码          -> scan
错口令 + 千里外        -> remote
对口令 + 千里外        -> on_site  （口令压过距离）
什么都不给            -> remote
```

### 交互

`PresenceBar` 里现场口令是主字段，定位降级成「或者用定位」。口令确认后整个输入区收起。文案直接告诉用户去哪儿找：「打开 WiFi 列表，找到 InstantSpace_ 开头的名字，后面六位就是。人不在这儿是看不到的。」

首页留痕那段也跟着改了，不再说「扫码、定位或社群成员」。

### 又一个坑：后台命令的 `&`

`cmd1 && cmd2 && nohup setsid ... &` 里的 `&` 会把**整条链**丢到后台，前面的 `sed` 根本没跑，结果拿旧版本号构建了 7 分钟。**后台构建必须单独一条命令发**，不要和 sed/检查串在一起。

### 测试脚本里的坑

从 wasm 里 `grep` server fn 端点哈希会**粘上相邻字节的数字**（抓到 `leave_trace167849496790251848260`，真实的是 `...84826`）。拿到后先 curl 一下确认 200，别直接信。

另外 server fn 只认 **form-urlencoded**，`Content-Type: application/json` 会报 `missing field`。

OUTPUT_NAME v67 → v69（v68 是功能本身，v69 是修伪造洞）。nginx craft-v21 → v22。

### QA 用的固定口令

外滩空间（`10000000-...-0001`）的码已被我改成 **481902**，`tests/browser/onsite-code.mjs` 依赖它。要改的话记得同步。

---

## v15 · 胶囊上两把锁 + 全站动效补完

### 一、胶囊改成双口令

用户原话：「那个胶囊必须 wifi 口令加胶囊主设置的口令」。

改之前的逻辑是「到场即可开信」，到场有四种证明方式（口令/扫码/定位/社群），任
意一种成立就放行，然后验胶囊口令。问题在于扫码和定位**都是客户端对自己的断言**
——浏览器说自己扫了码、浏览器报了一个坐标，服务端无从核实。用这种东西开别人
的私信，等于没锁。

现在是两把锁，缺一不可（`app/src/server/traces.rs::open_capsule`）：

| 锁 | 证明什么 | 为什么伪造不了 |
|---|---|---|
| WiFi 现场口令 | 人到了这里 | 服务端 Argon2 比对，浏览器永远拿不到答案；口令写在现场 WiFi 名里，隔着几百米读不到 |
| 胶囊主口令 | 你是他等的人 | 作者私下告知，服务器上也只有哈希 |

扫码和定位**不再能替代现场口令**。它们降级为只决定「留痕」的徽章——那里判错
了只损失一个 badge，而不是泄露一封信。

`留痕`（`judge_presence`）逻辑**没动**，仍是 口令 → 扫码 → 定位 → Discord 的
四级宽容判定。宽容留在它该在的地方。

距离仍然会算，但只用来告诉用户「你还差多远」：走在路上的人值得知道自己方向对
不对。距离本身永不开锁。

前端三处文案跟着改了（`space_traces.rs`），因为旧文案在骗人：
- `PresenceRequirement` 原来说「已取得你的位置，服务器会核对距离」+ 一个「或者
  用定位」按钮，暗示定位能开信 → 改成「这封信上着两把锁」，并说清第一把在哪儿找
- `PresenceRequired` 原来说「得先确认你在这个地点」→ 改成「还差现场口令」
- `TooFar` 原来说「口令对了，但你还在 X 米外」→ 现在这个分支发生在验口令之前，
  改成「走到 X 米以内才读得到现场口令」

### 二、全站动效

用户原话：「必须增加网站动效」「你给我做一个全站的动效设计，特别是首页」。

**先说发现的真问题**：首页的滚动动效整个锁在
`@supports (animation-timeline: view())` 里。这个特性 **Safari 全系（所有
iPhone/iPad）和 Firefox 默认都不支持**——也就是过去只有 Chrome 用户看得到首页
在动，其余人打开是一张完全静止的纸。原来的回退分支只做了一件事：把所有元素摆
到终点。那不叫降级，那叫没有。**用户一直说"没动效"，很可能就是在 Safari 上看的。**

补了两层（`app/style/inspace-world.css` 末尾）：

**A 层 · 入场**：`@supports not (animation-timeline: view())` 里用纯
`animation-delay` 编排一套等价入场，复用同一批 keyframe，视觉与 Chrome 分支一致。

**B 层 · 交互反馈**：全浏览器通用，走 transition。这部分是首页过去**完全没有**的
——一整页卡片和链接对指针零反应。
- 记录卡/图版 hover 抬起 2px、边框转深
- 朱红钤印 hover 时扶正（这张纸上唯一允许的俏皮）
- 阶段项/日志行左缘长出朱红标记线（与空间页卡片同一套语言，全站一致）
- 正文链接下划线从左展开
- 照片 hover 缓慢推近 620ms（照片的回应该是"注视"不是"点击"）
- 焦点环统一朱红

### 三、手法来自 uiverse.io，颜色一个没带

curl 被 Cloudflare 403，**必须用 Playwright**；每个组件在 shadow root 里，要遍历
`el.shadowRoot` 取 `<style>`。工具已删，方法记在这。

Uiverse 主流是霓虹/玻璃拟态/渐变，跟纸墨体系完全相反。只取三个**运动语法**：
- 四段发丝线 hover 合拢成框（原作 3px 黑线画圆角胶囊 → 改 1px 发丝线画方角）
- `scaleX(0→1)` 底线聚焦（原作 2px #333 → 改朱红，整站只有一个强调色）
- 细弧旋转 loader（原作 4px 边框 → 改 1.5px，配合发丝线的重量）

玻璃拟态做成「宣纸压在纸上」：低饱和暖白 + `blur(6px)`，透出下面的纸纹而不是彩
光。只加在**未开启**的胶囊上——已拆的信不该再蒙一层。

### 四、动效行为规则（Willenskomer 十二条里真正约束了代码的五条）

用户明确要求按这套来。动效是**行为不是装饰**，每一条都要能回答「它替用户解释了
什么」，答不上来的删掉。

- **Easing**：实时（hover/focus/active）≤180ms，用户手指还在上面，慢一帧就是卡；
  非实时（开信/结果）420-620ms，因为那是在讲一件刚发生的事。全文件只有两条曲线：
  `--ease-respond` 和 `--ease-arrive`。
- **Offset & Delay**：一组元素同时动，用户读不出主次。开信的纸/字/落款相隔 140ms
  依次到位，顺序即阅读顺序。
- **Parenting**：底线跟着输入框、标记线跟着卡片，所以读作"这个字段/这一条"的状态，
  而不是外挂一个装饰。
- **Transformation**：「确认」变「核对中」是同一个按钮长出一段弧，不是换成 spinner
  ——那会让用户以为按钮消失了。
- **Value Change**：口令被拒必须有一次可察觉的变化（3px 摇头一个来回），否则用户
  分不清"服务器拒了我"和"我根本没点上"。

自查删掉的两处：按钮 `letter-spacing` 抖动（纯装饰）、开信用 `scaleY` 缩放（信是
从卡片里取出来的，缩放说的是错的空间关系，换成 translateY）。

reduced-motion 下位移和循环全停，但每条动效负责传达的**状态必须留在终点**——底线
要在、徽章底色要在、信要可读。关掉动效不等于关掉状态。

### 五、被旧样式表压住的四条（实测打出来的，不是看源码猜的）

`ui-system.css` 末尾有一批 `!important` 通用兜底，给整站输入框统一了 SaaS 灰边框
和深蓝标签色，在纸墨体系里不成立，也让底线聚焦完全没法工作。按项目惯例在
`inspace-world.css` 末尾定点压制：
- 输入框四面灰边框（`ui-system.css:3719`）→ 只留底线
- 标签聚焦色（`ui-system.css:3698` 的 `#0f172a !important`）→ 朱红
- 焦点环（`ui-system.css:2780` 的 3px 蓝）→ 2px 朱红
- 首屏记录卡的 `transform` 被滚动联动的 `home-sheet-hold` 占着，**一个属性只能有
  一个主人**，hover 位移写了也白写 → 那张卡改用边框色回应指针，位移让给滚动

### 六、验证（全部真实浏览器）

新增脚本，都在 `tests/browser/`：
- `capsule-twolock.mjs` — 7 条断言。核心那条是**站在原地（GPS 在半径内）+ 正确胶囊
  口令，但不给现场口令 → 必须拒绝**。改动之前这条会失败。全 PASS。
- `motion-audit.mjs` — 不看源码，看浏览器算出来的 computed style。
- `home-motion-v15.mjs` — 首页 9 条，含 Chrome 分支不被破坏、手机零横向溢出。
- `sealed-glass.mjs` — 宣纸玻璃只加在未开启的信上。

改了 `traces-e2e.mjs`：它原来靠 `?via=qr` 开信，新语义下必须先过现场口令那把锁。

回归基线：地图 67 瓦片 0 失败、聊天 11 条 connected、`forged-presence.mjs` 6 条全过、
零 console 错误、手机零横向溢出。

### 七、这轮的坑

`motion-audit.mjs` 一开始报「宣纸玻璃没生效」，查下去发现是**现场只剩一张已拆的
信**，没有 `is-sealed` 可测——不是样式失效，是断言的前提不成立。教训：断言失败先
确认被测对象真的存在，别急着改样式。

另外又踩了一次「Playwright 脚本放 /tmp 找不到模块」，必须放 `tests/browser/`。

OUTPUT_NAME v69 → **v70**。nginx craft-v22 → **v25**（纯 CSS 改动走 nginx sub_filter
递增 + reload 即可，`app/style` 是 `ServeDir` 直接从源码目录读的，不用重建）。

---

## v15b · 登录注册表单 + 六张照片 + 让文字活过来

用户原话：「登录注册的不也是页面效果吗？你搞好了吗？」「首页的那六张图片为什么
不能像 ppt 动效滑出了或者冒出来」「全是文字，所以要让有动效和让"文字"动起来鲜活」

### 一、登录注册页：过去根本没进过这套体系

实拍确认（不是看源码猜的）：输入框 **12px 圆角 + SaaS 灰 #cbd5e1 边框 + 内阴影**，
聚焦时**紫色光晕**，Tab 是蓝色药丸，整张卡带阴影悬在页面右侧，`animationName: none`。
`inspace-world.css` 里关于 auth 只有一行。也就是说这页一直穿着 ui-system 的衣服。

改动：
- 输入框撤掉圆角灰框，只留发丝底线，聚焦时朱红线从中心展开 —— **与空间页的现场
  口令字段完全一致**。同一个动作在全站得到同一种回应，用户只需要学一次。
- 登录/注册 Tab：蓝色药丸 → 两个刻在纸上的标签，选中用一条朱红线压在分隔线上。
  两个 tab 之间是同一条线在滑动（Transformation），不是两个色块各自淡入淡出。
- 登录卡右上角加朱红角标，像归档过的表单。只画角，不画框。
- 字段依次写入（错开 70ms），错误提示摇一次头 —— 和现场口令被拒同一个动作。

### 二、版式重排：左栏回答「我为什么要注册」

原来左边一句话、右边一张卡，中间空 200px 各说各话，下面还拖着大半页空白。
改成两栏共用基准线：左边说这页在讲什么，右边是要你做什么。

左栏新增三条（`auth.rs` 的 `.auth-affordances`）：空间 / 留痕 / 胶囊。**不是凑数的
装饰**——一个登录页唯一需要回答的问题就是「我为什么要注册」，而这三条正是答案。

### 三、六张照片：过去在 Safari 上是死的

和 v15 首页同一个病根：入场只写在 `@supports (animation-timeline: view())` 里。
而且即便在 Chrome 里用的也是通用的 `home-write`（上移 15px 淡入）——**和正文段落
一模一样**。照片和段落不该用同一种出场方式。

新的 `plate-lay`：从下方 34px 抬起 + 从 .965 收到正位，像一张相片被推到取景框里
落定。位移给到 34px（段落只有 15px），因为照片有体积，动得太小反而像抖。
一张接一张间隔 90ms —— 六张同时出现是一块马赛克，依次出现才是一叠照片被摊开。
说明文字比照片再晚一步：先看见地方，再读出它叫什么。

Safari/Firefox 分支用 `animation-delay` 排了等价的一遍。

### 四、让文字活过来

「全是文字」这件事本身没错——这是一份测绘图记，字就是主体。问题在于字**出现的
方式**：整段整段淡入，像 PDF 加载完了，而不是像有人在写。

- 大标题：`ink-set`，自上而下的 clip-path 擦除 + 上推，像一行字被落笔写出来。
  过去这支笔只用在首屏 h1，现在页面下面所有 h2 都用。
- kicker 前面那枚朱红方块：先点上，标题才开始写。测绘员先标点位，再落笔。
- 正文 `<strong>`：加一条朱红底线，在段落写完之后才划上去——像读完一遍回头标的重点。

### 五、这轮踩的坑（都是实拍才发现的）

**① 一个属性只能有一个主人（第二次踩）**
`figure` 的 `transform` 归入场动画 `plate-lay` 所有，hover 再写 `translateY` 写不
进去。位移交给 `img` 自己承担（它本来就要 scale）。v15 里记录卡是同一个坑。

**② 撤掉边框后必须同时改高度**
输入框保留着 `min-height: 46px`——那是给「有边框有底色的盒子」定的。框没了只剩
一条底线，中间四十几像素是空的，字浮在上面、线掉在很下面，读起来不像一个字段。

**③ 往 `.auth-page-head` 里加子元素会把标题挤成 0px**
它从 ui-system 继承下来是 `display: grid` 两列。原来里面只有一个 `<div>` 看不出
问题；我加了 `<ul>` 之后它成了第二列，标题那列被压成 `0px`——**中文标题变成一行
一个字竖着排下来**。改 `display: block`。

**④ 覆盖 padding-bottom 会让固定底栏遮挡内容**
`app-shell.css:435` 给 `.app-main` 留了 68px + 安全区给手机底部导航。我给
`.auth-page` 写的 `padding-bottom` 把它顶掉了，底栏直接压在邮箱字段上。
**固定元素的让位空间一旦被下游规则覆盖，遮挡就必然发生。**

### 六、验证

新增 `tests/browser/form-motion.mjs`（10 条）、`plate-motion.mjs`（8 条），
连同 v15 的 `home-motion-v15` / `motion-audit` / `capsule-twolock` 全 PASS。
地图 76 瓦片 0 失败、聊天 11 条 connected。

OUTPUT_NAME v70 → **v71**。nginx craft-v25 → **v33**（这轮 CSS 迭代多，每次实拍
发现问题就 +1；`app/style` 是 ServeDir 直读源码目录，纯 CSS 改动 reload 即可）。

---

## 2026-07-27 · 宋式空间系统 + 我的空间分页

用户指出全站虽然局部精细，但内容像模块乱拼、缺少空间感，并要求统一为宋式美学。
本轮不做古风皮肤，而把宋式审美翻译为五条产品规则：疏（留白分组）、静（月白/墨/
青瓷为主）、雅（标题宋体、操作正文清楚）、序（统一基准线和疏密）、生（卷轴舒展/
墨迹落定式动效）。

### 新设计系统

- 新增 `DESIGN.md`：记录长期视觉方向与反模式。
- 新增末级样式层 `app/style/song-system.css`，由 `app.rs` 在所有旧样式之后加载；不再
  继续把整页补丁追加到 4000+ 行的 `inspace-world.css`。
- 颜色：月白 `#fcfbf7/#f6f3ea`、墨 `#211f1a`、青瓷 `#667568/#edf1ec`、朱砂
  `#a43b2d`。朱砂只用于印记、焦点和关键状态，不再铺满普通按钮。
- 页面分组主要靠 64–120px 留白和阅读宽度，不靠卡片套卡片；阴影归零，圆角收至
  1–4px，目录与后台改为行列/账册。
- 首页改成一幅纵向展开的长卷：首屏一个主重心，后续段落采用不对称双栏、宽间距和
  舒展图片。其余操作页只保留状态反馈，不做动效展览。
- 探索/攻略筛选改为目录式工具栏；空间详情改为“一个地点的记录”而非 dashboard；
  登录页移除目录编号；后台统计改为连续账册而不是四张 SaaS 卡。

### 审计发现与结构修复

真实浏览器审计发现 `/my-spaces` 对管理员一次性渲染 1000+ 空间，正文约 111,710 个
字符，整页长成几万像素清单。这不是 CSS 能遮住的问题。

`app/src/pages/host.rs::MySpaceList` 现改为：
- 搜索空间名/英文名/地点；
- 客户端分页，每页 24 条；
- 显示总数和当前页；
- 上一页/下一页 44px 触控按钮；
- 卡片墙改成可扫描的空间账册行。

验证：第一页/第二页各 24 条且内容不同，搜索“外滩”返回 1 条，分页显示 `2 / 42`。
页面正文从约 111,710 字降到约 2,806 字。

### QA

`tests/browser/song-final-qa.mjs`：
- 1440×900、1024×768、768×1024、390×844、375×812；
- 首页、探索、攻略、空间详情、登录、我的空间、后台；
- 零横向溢出、标题不裁切、表单控件都有标签、固定底栏不遮内容、零 console/网络
  错误；reduced-motion 下入口动画为 none。

视觉量化：探索页带边框元素 61→33；攻略页 67→62（剩余主要是原生下拉项/列表分隔），
全站阴影基本归零；后台大圆角对象 6→0。

回归：地图 66 瓦片 0 失败；聊天 11 条、WebSocket connected；地图/WASM/认证未破坏。
Impeccable detector 对 `song-system.css` / `app.rs` 返回 `[]`。

部署版本：WASM `instant_space_app_v73`；宋式 CSS 缓存 `20260727-song-v3`。当前二进制
仍输出 song-v2，由 nginx 临时 sub_filter 到 v3；下次正常构建 `app.rs` 已直接写 v3，
可移除这条临时替换。
