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

## 2026-07-27 — iPad 横屏 / 中间断点修复（song-v5）

- 用户实机截图暴露 1100–1366 CSS px 区间：桌面侧栏已展开，但首页仍使用宽屏双栏和基于全视口的字号/间距，`.survey-field-head h2` 被压成一字一行；旧 QA 只检查首个 `h1`，因此漏报。
- `app/style/song-system.css` 新增 1100–1399px 内容驱动断点：首页 hero、passage、field、keep 在侧栏后的有效画布内重组为单栏；标题、登录页及共享页面标题改用适合有效画布的字号/间距。
- 移除旧版 `.survey-field-head` / `.survey-keep-head` 的 34–36em 宽度瓶颈；1440px 以上恢复安全双栏，标题轨最小 320px。
- 样式缓存版本升级到 `20260727-song-v5`；线上 nginx 已将旧 v2/v3/v4 HTML 引用替换到 v5，无需 Rust release 构建即可生效；`app/src/app.rs` 同步 v5 供下次构建。
- 新增 `tests/browser/tablet-breakpoint-audit.mjs`，检查 1024、1100、1140、1180、1194、1240、1280、1320、1366、1440 下首页/探索/攻略/空间/登录/工作台/后台的所有 h1/h2、网格和横向溢出。
- QA：上述全部宽度和路由无 5 行以上异常标题、无横向溢出；`song-final-qa.mjs` 全通过；地图 66 个瓦片请求成功、0 失败；聊天 WebSocket `connected`；`cargo check -p instant-space-app` 通过。

## 2026-07-27 — 首页中文文字编排修正（song-v7）

- 实机反馈确认根因不只是断点，而是首页把中文完整句子当海报字处理：超大字号、窄标题栏、标题与解释分居远距离双栏，造成竖排感和语义断句失控。
- `.survey-passage`、`.survey-field-head`、`.survey-keep-head` 改为真正的编辑阅读单元：眉题 → 1–2 行主标题 → 紧随说明；桌面下方内容再展开，手机重组为单列。
- 中文展示标题使用受控字号 `clamp(2.85rem, 3.45vw, 3.75rem)`、约 14em 行长、`word-break: keep-all`；不再用 4.5vw/4.8rem 超大字承担布局。
- `survey-passage` 的三个回答改为标题说明下方三栏，手机单列；标题不再塞进窄左栏。
- 线上缓存升级 `20260727-song-v7`。1024–1440 中间宽度审计、桌面/iPad/手机全站 QA、reduced-motion、无障碍标签与横向溢出检查全部通过；Impeccable detector 无未解释告警。

## 2026-07-27 — 地图 SPA 懒加载修复、控件迁移与首页编辑器升级（v74）

- 地图持续显示“正在加载”的根因不是瓦片服务：`app/src/map_boot.js` 只在首次页面启动时判断地图路由；从首页经 SPA 导航进入 `/inspace/map` 后新出现的 `#map` 未触发 MapLibre 加载。现通过 `hasMapSurface(root)` 识别后续挂载的正式地图，并由 `crates/map-ui/src/maplibre_shim.js` 主动请求 loader；失败时结束无限 RAF 重试并展示可恢复错误状态。
- 地图右上角整组“返回页面探索 / 道路 / 深色 / 3D 地球”已从地图画面移除。道路/深色与 2D/3D 改到全局左侧导航的地图专属工具组；手机端位于导航抽屉，选择后自动关闭，不再遮挡地图探索。状态由 `AppState.map_style` / `map_projection` 共享。
- 首页编辑器从长表单升级为页面编辑器：顶部 sticky 命令栏（草稿/线上、预览、保存、发布）+ 左侧页面结构 + 中间折叠属性区 + 右侧实时预览；支持桌面/手机与中英文预览，预览覆盖 Hero、用户旅程、攻略价值和主理人 CTA；手机端重组为单列。
- 部署版本：WASM/loader `instant_space_app_v74`，生产二进制 `/usr/local/bin/instant-space-app` 已替换并重启 `instant-space-rust`。
- 地图真实浏览器验证：桌面与 390px 手机均已挂载 MapLibre、style loaded、loading 隐藏、无地图悬浮模式控件、左侧/抽屉工具可用、无横向溢出；一次健康检查 67 个瓦片请求成功、0 失败、11 个聚合点。
- 首页编辑器真实浏览器验证：桌面/iPad/手机均显示 6 个编辑区块，无横向溢出、无 console error；手机触控目标均不小于 44px。
- `tests/browser/song-final-qa.mjs` 最终全通过：桌面、iPad、手机、小屏手机及 reduced-motion 均 PASS。

## 2026-07-27 — 管理控制台首页页面编辑器 v3/v4

用户指出上一版仍只是“目录 + 长表单 + 小预览”，不是真正的页面编辑器。本轮按成熟视觉编辑器的任务结构重构 `/inspace/admin/home`：

- 工作区改为真正三栏：左侧页面结构树、中间主画布、右侧上下文属性检查器；画布成为视觉中心，不再让长表单主导页面。
- 结构树支持四个首页区块的选择、显示/隐藏和上移/下移；顺序直接写回已有 `order` 字段，无需改变数据库契约。
- 点击画布区块会打开对应属性面板；属性面板只显示当前区块内容，中文/英文用语言切换编辑，避免同时铺开两套字段。
- 画布支持桌面、平板、手机三种宽度和中英文实时预览；使用容器查询单位控制画布标题，不再错误地按浏览器视口放大。
- 顶部命令栏新增未保存状态、撤销修改、保存草稿、发布和查看线上；保存/发布继续沿用现有版本化后端。
- 页面设置独立为主题与版式、导航与 SEO、发布历史；SEO 面板提供搜索结果摘要预览。
- 手机端将结构树重组为横向可扫的区块轨道，画布和属性检查器依次向下排列；所有实际可见控件通过触控尺寸检查。
- 视觉仍遵循宋式空间系统：月白纸面、墨色、青瓷、克制朱砂，不使用卡片套卡片或装饰性渐变。

部署与验证：
- WASM/loader 升级为 `instant_space_app_v75`；生产二进制已部署并重启。
- `backoffice.css` 缓存版本为 `20260727-editor-v4`；当前生产二进制输出 v3，由 nginx 临时替换为 v4，源代码已直接写 v4 供下次构建。
- 管理员真实浏览器 QA：1440×900、1024×768、390×844 均显示 4 个结构区块、实时画布和属性检查器；零横向溢出、零未标记表单控件、零 console/network 错误、无过小控件。
- 交互验证：选择区块、设备切换、隐藏/恢复区块、编辑后脏状态、撤销恢复全部通过；未在 QA 中触发真实发布，避免产生无意义线上版本。
- 全站 `song-final-qa.mjs` 在桌面、iPad 横竖屏、手机、小屏手机及 reduced-motion 下最终 ALL PASS。
- `tests/build/wasm-build.mjs` 原先硬编码 v64，已改为从 `scripts/build-wasm.mjs` 自动读取 OUTPUT_NAME，避免后续版本号升级导致假失败。

## 2026-07-27 — 首页标题显式换行与审计表迁移漂移修复（v76）

- 首页编辑器“主标题”继续使用多行 textarea，并增加“按 Enter 控制换行”的明确提示；编辑器画布 `.canvas-hero h2` 与公共首页 `.survey-hero h1` 均使用 `white-space: pre-line`，保存的 `\n` 不再被 CSS 折叠。
- 管理后台审计资源移除 `list_audit_log().await.unwrap_or_default()`：真实空数据仍显示“暂无操作记录”，数据库或 schema 错误改为 `role=alert` 的明确错误，不再伪装为空。
- 新增幂等修复迁移 `20260727000200_repair_admin_audit_log.sql`，恢复生产 `admin_audit_log` 表和三个索引；生产验证表存在、记录数为 0、新旧两条迁移均 `success=true`。
- 首次部署揭示 SQLx 新增迁移文件未触发 `instant-db` 重编译；新增 `crates/db/build.rs` 的 `cargo:rerun-if-changed=migrations`，保证以后新增 SQL 会进入嵌入式 migrator。
- 迁移执行又揭示应用数据库角色缺少旧 `users` 表的 `REFERENCES` 权限；已只授予该最小权限，外键创建成功，服务恢复 active，`/health` 与 `/ready` 均通过。
- 部署版本：WASM/loader `instant_space_app_v76`，`backoffice.css?v=20260727-editor-v5`，`song-system.css?v=20260727-song-v9`。
- 真实浏览器专项验证：1440×900、1024×768、390×844 下，公共标题与编辑器预览均为 `white-space: pre-line`，输入两行后预览保留换行，无横向溢出、无裁切、无 console error。全站 `tests/browser/song-final-qa.mjs` 桌面、iPad 横竖屏、手机、小屏手机和 reduced-motion 全部通过。

## 2026-07-27 — 非商业许可证

- 仓库新增 `LICENSE`，采用 PolyForm Noncommercial License 1.0.0。
- 必要声明：Copyright 2026 InSpaceOS；未经许可方事先书面授权，禁止商业使用。
- Workspace 及所有 Rust package 均声明 `PolyForm-Noncommercial-1.0.0`，并在 README 中加入中英文许可证说明。

## 2026-07-27 — 胶囊写入双现场验证、全国省级空间与主理人招募（v78）

### 胶囊写入
- `seal_capsule` 不再只检查登录：埋胶囊必须同时通过空间 Wi-Fi 名称中的现场口令和浏览器 GPS 半径验证；只满足一项、没有定位或距离过远都不能写入。
- 正常拒绝使用 `CapsuleSealResult` 结构化返回，不再把“距离太远”当 HTTP 500；服务端仍是最终权限边界，前端 disabled 只负责引导。
- 胶囊编辑器新增两项现场证明清单；扫码不会再隐藏现场口令输入；“或者用定位”改为中性的“验证当前位置”。
- `tests/browser/capsule-seal-presence.mjs` 覆盖：无证明、仅 Wi-Fi、Wi-Fi+近场 GPS、Wi-Fi+远场 GPS；`capsule-twolock.mjs` 同步适配写入规则。两组均 ALL PASS。

### 中国省级内容补全
- 新增幂等脚本 `scripts/seed/seed_china_provinces.py`，从生产 `geo_places` 的真实城镇坐标生成编辑部基础空间，不虚构著名景点事实。
- 中国 34 个省级地区（含港澳台）各写入 50 个空间和 50 份一一绑定的攻略，共 1,700 + 1,700；逐省 SQL 核验全部为 50/50。
- 这些空间全部 `host_user_id = NULL`，标记“主理人招募中 / Host wanted”；系统管理员负责先点亮，等待当地人认领和补充真实路线、现场变化与地方故事。
- 生产总量更新为 Spaces 2,714（active 2,702）、Guides 2,701。地图健康检查 MapLibre style loaded、11 个聚合点、59 个瓦片成功、0 失败。

### About 与主理人招募
- 新增 `/inspace/about`：解释“地图负责到达，inspace 负责体验”、三层能力、空间主理人职责，以及压缩后的创始人寄语。
- 全局侧栏底部增加“关于 inspace”；首页末段改为公开招募空间主理人，并链接 About 招募章节。
- 迁移 `20260727000300_home_host_recruitment.sql` 同步更新当前草稿和已发布首页文案。
- About 使用独立 `about.css`；第一次截图发现桌面 2px 溢出和深色段落对比度不足，第二轮修复后桌面/平板/手机均无溢出、按钮可见、触控目标不小于 44px。

### 部署与验证
- 生产版本：WASM/loader `instant_space_app_v78`，`about.css?v=20260727-about-v2`，`inspace-world.css?v=20260727-capsule-v18`。
- `/health` 与 `/ready` 正常；迁移 `20260727000300` success=true。
- `song-final-qa.mjs` 已加入 About，桌面、iPad 横竖屏、390/375 手机和 reduced-motion 全部 ALL PASS；无横向溢出、标题裁切、未标记字段、console/network 错误。

## 待办路线图（2026-07-27，已与用户敲定，尚未动代码）

> 详见 `docs/SPACE_DETAIL_AND_API_PLAN.md`。执行顺序 **2 → 4 → 3 → 1**。

核心认知：**空间 = 任何真实地点的数字入口**（公司/餐馆/公园/景点都能建），不只旅游景点。之前文案写不好就是没抓住这点。

1. **对外 API（作者自用，最后做，别忘了！）**：带 API Key 鉴权的 REST（`/api/spaces`、`/api/guides` 增改查），让作者的 AI agent 程序化建空间/写志/管空间。这轮只做 API+鉴权，不接模型。暂不开放给普通用户。
2. **详情页重构（先做）**：「攻略」→「志」（英文暂 Guide，待确认）。详情页做成**卡片墙**（点开才进，非长滚动）：顶部空间头 + 「关于」区（简介卡/主理人+发展史卡/故事卡）+ 志卡 + 讨论卡。志的板块按**可自定义的空间类型**动态适配，故事暂不并入讨论。
3. **首页加轮播**：用公司/餐馆/公园/景点等示例讲清「创建空间为了什么」。
4. **文案重写**：首页+About，从「旅行攻略」扩展到「给在乎的真实地点建可进入的数字空间」，保持宋式美学。

已完成背景：反向地理选点已修（简体中文 + 台湾/港澳归正中国）；中国种子已换 Wikidata 真实景点（1062 招募空间，南昌可搜到滕王阁）。WASM 版本推进到 v80。

## 主理人认领 + 简介/主理人可编辑（2026-07-28，已上线 v84）

用户反馈的逻辑缺口：卡片墙写“认领后可写简介”，但代码里既没有“认领”动作，认领了也没有改简介的入口。现已补齐闭环，三块内容均可编辑运营。

- 认领方式（用户拍板）：申请 → 管理员审批，防止故宫等热门点被抢占。主理人卡片只加“寄语”，不单列联系方式。
- 迁移 `20260728000100_host_claims_and_bio.sql`：`spaces` 加 `host_bio_zh/en`；新表 `space_host_claims`(space_id,user_id,message,status,created_at,decided_at, UNIQUE(space_id,user_id))，pending 部分索引。
- DB (`crates/db/src/spaces.rs`)：`UpdateSpaceInput` 扩 `custom_type/description_zh/en/tag_zh/en/host_bio_zh/en`；`update_host_space`、`get_space_detail` 同步；新增 `apply_host_claim`(仅未认领空间受理，ON CONFLICT 重置)、`host_claim_status`、`list_host_claims`、`approve_host_claim`(事务：赋 host_user_id + 本条 approved + 同空间其余 pending 置 rejected)、`reject_host_claim`。
- domain：`SpaceDetail` 加 `host_bio_zh/en`；`admin::HostClaimApplication`。
- server：`SpaceDetailView` 加 host_bio；`update_my_space` 扩 7 参（host.rs 调用点同步，19 参加 `#[allow(clippy::too_many_arguments)]`）；新增用户态 `apply_host_claim`/`my_host_claim`（返回 `HostClaimState` 枚举 Anonymous/None/Pending/Approved/Rejected/AlreadyHosted）；admin `list/approve/reject_host_claim`（审批写 audit）。
- 前端：`space.rs` `SpaceHostPanel` 加 `space_id`，已认领显示寄语，招募中渲染 `SpaceHostClaim`（按 claim 状态显示登录/申请表单/审核中）；`host.rs` `ManageSpacePanel` 拉 `get_space_detail` 预填，编辑表单加 简介/自定义类型/标签/主理人寄语 字段；新页 `admin_claims.rs`（路由 `/admin/host-claims`，侧栏“认领”）。CSS 追加 `.space-host-bio/.space-host-claim-*/.admin-claim-note`。

### 部署踩坑（重要）
- 迁移首次失败：`ALTER TABLE spaces` 报 `must be owner of table spaces`。根因：服务以 `instant_space` 连接，但表属主是 `postgres`，服务因此崩溃重启循环。
- 修复：`sudo -u postgres psql instant_space_rust` 把 public schema 全部表/序列 `OWNER TO instant_space`（`REASSIGN OWNED` 思路），重启后迁移自动补跑成功（migrations=17，`IF NOT EXISTS` 幂等）。以后新迁移改 spaces 表不会再报属主错。
- 生产版本：WASM `instant_space_app_v84`，`inspace-world.css?v=20260727-cardwall-v22`。cardwall QA 六视口 ALL PASS，滕王阁招募态认领入口截图正常。

### 路线图进度
- 第 2 步（详情页卡片墙）完成，并额外补齐认领+可编辑闭环。剩余：4 首页/About 文案（About 已改）、3 首页轮播、1 对外 API（最后）。

## 2026-07-28 · taste-skill 全站审计与首轮修复

- 使用 `design-taste-frontend`、`inspace-design-engineer`、响应式、无障碍和 Playwright 质量门扫描生产站；报告在 `output/playwright/taste-audit/report.json`，覆盖首页、About、探索、攻略、攻略详情、地图、两类空间详情、聊天、登录、用户/管理员登录门，视口为 1440×900、1024×768、390×844。
- 全站共同通过项：测试路由无横向溢出、无控制台错误、图片 alt/表单标签无严重缺失；地图画布和聚合点正常，OpenFreeMap 的 `ERR_ABORTED` 是 MapLibre 主动取消旧瓦片请求，不是地图加载失败；reduced-motion 下首页动画为 0。
- 首轮修复：`song-system.css` 最终接管首页 Hero，解决后加载的旧 `!important` 把标题挤成桌面 5 行、手机 4 行的问题；线上实测桌面/iPad/手机均为 2 行且 overflow=0。
- 触控修复：登录输入框、攻略详情返回入口、地图聚合点均达到至少 44px。
- HyperFrames 专项证据在 `output/playwright/hyperframes/`：桌面和手机四幕均可正向切换，反向滚动可回放，active frame 与 active visual 一致，无横向溢出；reduced-motion 退化为普通文档流。
- 当前结构性优先级：Explore（边框 76、嵌套边框 50、手机小字 51）→ Guides（边框 58、嵌套 50）→ About/攻略详情的长标题与破折号文案 → 登录/全局 shell 的次级触控细化。探索和攻略需要结构重构，不能继续靠追加装饰 CSS：删除筛选容器套列表容器的双层框，分类改文本索引/分段线，列表对象只保留一层边界。
- 本轮未做 release/WASM 构建；`cargo check -p instant-space-app` 通过。通过 Nginx 静态样式版本替换上线 `inspace-world ... v3` 与 `song-system ... v11`，只 reload Nginx，未重启应用或其他服务。

## 2026-07-28 · 探索与攻略从修复升级为地点索引

- 新增专用样式层 `app/style/directory-system.css`，由 `app/src/app.rs` 最后加载；它只接管 Explore/Guides，避免继续在通用旧 CSS 里堆紧急覆盖。当前生产缓存版本 `20260728-directory-v3`。
- Explore 从“筛选卡 + 列表卡”升级为地点索引：主叙事改为“从一个地点，进入它的空间”，搜索具体地点优先，类型作为第二层缩小范围，结果行只保留名称、类型、地点、访问状态和进入动作；创建 CTA 明确为“为熟悉的地点建空间”。
- Guides 升级为地点阅读索引：标题强调“先选一个地方，再读那里留下的攻略”，明确攻略应在用户管理的空间内创建；搜索之后按省份→城市→区域→地点逐级筛选，新增可真正复位所有 select DOM 状态的“清除全部”；攻略行改为标题、地点、阅读动作的编辑式索引。
- 数据/后端契约保持不变：Explore 20 条/页、Guides 24 条/页，既有服务端分页、搜索、类型/地区筛选与路由全部保留。
- 生产已部署 WASM/SSR `instant_space_app_v87`。本轮只做了一次 release（8m27s）和一次 WASM（3m09s）构建；之后的两次视觉修正仅通过 CSS 缓存版本与 Nginx reload 上线，没有重复构建。
- 最终浏览器 QA：1440×900、1024×768、390×844 均 overflow=0、console/network 0；Explore 嵌套边框 50→14（手机 41→11），桌面小字 47→7；Guides 嵌套边框 50→17（手机 41→14）；两页 H1 手机均 2 行、visible em dash=0、reduced-motion 动画=0。搜索“南昌”返回 18 个空间；类型筛选/清除、四级地区筛选/清除、空间行和攻略行点击跳转均验证通过。
- 最终证据：`output/playwright/directory-upgrade/final-report.json` 与同目录最终截图。
- 仍需单独处理的数据质量：部分历史批量导入的攻略标题/地点本身错误或过于泛化，这不是本次 UI 索引重构造成的；应在数据治理模块修正，不能用前端隐藏。

## 2026-07-29 · Taste 升级：空间详情地点索引与 About 叙事页

- 本轮继续严格使用 `design-taste-frontend`：Design Read 为“面向真实地点访客与未来主理人的品牌叙事和空间详情重构，宋式编辑感、真实地点沉淀、克制且有目的的动效”；参数为变化度 8、动效 6、密度 3。
- 空间详情不再展示五个等权大卡片。`SpaceCardWall` 已改为“地点入口索引”，顺序固定为简介 → 主理人 → 故事 → 空间志 → 讨论；第一条简介承担更完整解释，其他入口用墨线分隔的编辑式行，不再卡片套卡片。讨论仍跳独立路由 `/inspace/spaces/:id/chat`。
- 新增 `app/style/space-experience.css`，作为最后加载的空间详情专用样式层。分享和社群仍在桌面右侧工具轨，iPad/手机下落到索引之后；旧分享、二维码、社群、认领、故事胶囊和空间志业务契约均未改变。
- 简介面板的类型、位置、标签、地点说明从嵌套 fact cards 改为规则线信息行；面板返回按钮实测 44px，打开面板使用 300ms 状态转场，`prefers-reduced-motion` 下完全关闭。
- About 保持整页统一月白纸主题，取消中途深色主题翻转、`01/02/03` 编号和装饰点；新增真实地点图片窗口、首屏双 CTA、无编号主理人职责和创始人寄语。可见文案中的 em dash / en dash 为 0。
- About 的 scroll-driven 动效只保留轻微位移，不再降低文字 opacity，避免再次出现“透明灰字”；桌面/iPad H1 为 2 行，390px 手机为 3 行且无裁切。正文小于 12px 的可见文字为 0。
- 生产版本：SSR/WASM `instant_space_app_v88`，About CSS `20260729-about-v4`，空间体验 CSS `20260729-space-experience-v1`，migrations=19。仅执行一次 release（11m07s）和一次 WASM（4m03s）构建，健康检查与 ready 均正常。
- 因边缘 CSS 缓存为 4 小时，Nginx 的既有 cache-buster sub_filter 增加 `20260729-about-v3` → `20260729-about-v4`，确保普通刷新立即取得修正版；只 reload Nginx，未重启无关服务。
- 最终 Playwright QA 覆盖 1440×900、1024×768、390×844，以及故宫 hosted / 滕王阁 recruiting 两种空间：全部 overflow=0、console/network=0；简介、主理人、故事、空间志面板和返回全部通过；讨论路由通过；reduced-motion 动画全部为 none。
- 证据位于 `output/playwright/taste-space-about-final/final-report.json` 及同目录桌面、iPad、手机截图。`tests/browser/space-cardwall-qa.mjs` 已同步新 `.space-entry-row` 结构。

## 2026-07-29 首页 v7 中宽屏修正
- 用户截图指出首页在中宽屏/平板宽度下像单栏文档：左侧堆文字和大斜图，右侧大面积空白。
- 按 taste skill 收口：`app/style/home-reframe.css` 追加 v7 断点。
  - 901-1099px 保持左右首屏构图，图片在右侧，不再塌成单栏。
  - 721-900px 改成居中单栏，控制宽度与图片尺寸，避免内容靠左和右侧空白。
  - 首图减小失衡感，保留轻微倾斜但不再像素材硬贴。
- `app/src/app.rs` 首页样式版本推进到 `20260729-home-reframe-v7`。
- Nginx sub_filter 已把旧 `home-reframe` v1-v6 映射到 v7；若边缘缓存仍吐旧 HTML，可用带 query 的 URL 或 no-cache 验证。
- 同轮修复移动端 topbar 搜索：`app/style/app-shell.css` 恢复手机 placeholder，避免顶部出现空白长框；样式版本 `20260729-shell-search-v4`。
- Playwright 截图输出：`output/playwright/home-v7-responsive/`。

## 2026-07-29 首页 v10 去除平板/中宽屏大空白
- 用户反馈 v7/v8“改了跟没改一样，还是有大空白”。复查发现根因不是单纯断点，而是旧样式的列间距/更高权重规则继续生效，把右侧图片列压到约 300px，造成右边视觉空白。
- `app/style/home-reframe.css` 追加 v10 高权重覆盖：`body .inspace-home .survey-hero` 在 760-1180px 使用 `grid-template-columns: 42% 58%`，`gap/column-gap/row-gap: 0`，右侧图片占 58% 舞台。
- 760-900px 使用 `45% 55%`，避免中宽屏重新变成左侧文档流。
- `app/src/app.rs` 首页样式版本推进到 `20260729-home-reframe-v10`，Nginx sub_filter 映射旧 v1-v9 到 v10。
- Playwright 计算值：1024px 由旧 `499px 300px / gap 61px` 变为 `365px 504px / gap 0`；900px 变为 `342px 418px / gap 0`；overflow=0。
- 截图目录：`output/playwright/home-v10-no-blank/`。

## 2026-07-29 — 首页尺寸收敛 + 宋式美学执行准则（v6）

- 用户明确：上一轮问题不是热门空间卡片，而是首页「空间示例」轮播图和主标题同一视觉等级过大；已上线 `home-discovery-v6`。
- 首页尺寸：桌面主标题上限 `6.2rem → 5.05rem`；轮播图片最高 `640px → 380px`；手机轮播图 `252px → 200px`，手机标题上限 `3.25rem → 2.75rem`。`1440×900 / 768×1024 / 390×844` 无横向溢出、无 console/network error；轮播隐藏页不可获得键盘焦点。
- 平板热门空间：宽度 ≤900px 改为规则双列，避免桌面主卡跨行造成棋盘式空洞；奇数末项跨整行。手机仍保持横向滑动。
- 用户认可的宋式美学不是泛黄、大留白或仿古装饰。后续设计遵循：
  - 月白/米白为底，墨黑建立阅读秩序；雨过天青只作为低饱和层次，朱砂仅用于关键行动/状态。
  - 留白必须服务层级、阅读或转场，不能是无意义空洞。
  - 一屏一个视觉主角，图片、标题、正文严格递减；采用“一角半边”、虚实相生，而非均分堆叠。
  - 通过真实材质、细线、比例、字重表达质感；避免卡片嵌套、杂色、渐变和装饰性动效。
  - 动效应克制地解释状态/层级（如卷轴展开、墨色渐显），支持 reduced-motion，禁止抢内容。
- 资源：主机为 4 核。用户允许构建使用约 80% CPU；后续默认 `CARGO_BUILD_JOBS=3`（约 75% 并发），不要重复并行启动 cargo。当前 release 的单大 crate / 链接阶段并不一定随 `-j` 线性加速。

## 2026-07-29 — 首页宋园点景（v7）

- 用户要求增加园林装饰。首页 Hero 已增加低对比度的纯 CSS/SVG 装饰：月洞门、竹影和水纹；仅作为背景点景，不承载信息，不影响键盘/点击，不引入额外图片请求或装饰性动画。
- 响应式：桌面右侧完整点景；平板缩小并降低透明度；手机向右收边、透明度降为 `.33`，标题/正文始终在上层可读。
- 部署：`home-discovery-v7` 已发布，服务健康检查 `ok/ready`；1440×900、768×1024、390×844 均无横向溢出、无 console/network error，轮播隐藏链接不可聚焦。

## 2026-07-30 — 全站宋式文字色板（song-colour-v12）

- 用户指出“文字改色”要求的是全站，而不只是轮播。`song-system.css` 现建立全局“墨分五色”语义：
  - 墨黑：主标题与长文阅读；深墨灰：正文；石灰：地点、时间、说明；水青：路径/链接；青瓷：分类、标签、在线状态；梅子青：已发布/通过/成功；朱砂：风险、拒绝、删除及关键注意。
- 同时将旧 `--color-*` 遗留语义 token 映射至宋式 token，探索、空间详情、攻略、聊天、空间管理、登录与后台都会继承，不再回退蓝紫 SaaS 色。
- 发布版本：`song-system.css?v=20260729-song-colour-v12`，首页窗景 `home-discovery-v8` 一并发布。
- QA：首页、探索、空间详情、登录、我的空间、后台均无横向溢出、无 console/network error；四种窗形在桌面与手机轮播均可切换，隐藏轮播链接不可获焦。

## 2026-07-30 · 攻略搜索/国家筛选 + 作者 Agent REST API 已上线

### 用户原意（再次确认，防止忘记）
- `memory.md` 里说的“对外 API”不是 `/inspace/api` 的 Leptos Server Functions。
- 它是**作者自用 AI Agent REST API**：作者以后用自己的 Agent 程序化建空间、写志、管理空间；这轮只做 API 与鉴权，不接大模型、不开放给普通用户。

### 攻略目录修复
- 迁移 `20260730000100_guides_country.sql`：`guides` 新增 `country`，2063/2063 篇从绑定的 `spaces.country` 完整回填，索引 `guides_country_idx`。
- `GuideSummary`/`GuideDetail` 增 `country`；新建/更新志时 country 从绑定空间派生，避免 Agent 或用户填出不一致国家。
- 攻略页新增国家下拉（China/Egypt/France/Italy/Japan/Spain/Thailand/Türkiye/United Kingdom/United States）。
- 搜索从整句 `ILIKE` 改为拆词后 AND：每个 token 必须命中标题/地点层级/国家/摘要/正文之一；生产验证 `南昌 滕王阁` 精确返回 1 篇「滕王阁」。
- 攻略详情本身 SSR 一直正常；客户端“点不开”的实际根因是服务端 DOM 已更新但线上仍加载旧 WASM v89，触发 Tachys hydration panic。已构建 hydrate WASM `instant_space_app_v90`，Nginx 暂用精确 sub_filter 将公开 HTML 的 v89 loader 改写为 v90；源码 `app/src/main.rs` 与 `scripts/build-wasm.mjs` 均已推进到 v90，下一次正常 server release 后可移除这条临时改写。

### Agent REST API
- 新增迁移 `20260730000200_agent_rest_api.sql`：
  - `agent_api_keys`：绑定 user、`key_prefix`、Argon2 `key_hash`、scopes、每分钟限流、撤销/最后使用时间。
  - `agent_api_audit_log`：key/user/method/path/status/target/remote_addr/时间。
- 新增 `crates/db/src/agent_api.rs`：Key 查询、分钟计数、审计、空间/志归属检查。
- 新增 `app/src/agent_api.rs`，独立 Axum JSON REST，不复用 Leptos Server Function 协议：
  - `GET/POST /api/spaces`
  - `PATCH /api/spaces/:id`
  - `GET/POST /api/guides`
  - `PATCH /api/guides/:id`
- 鉴权支持 `Authorization: Bearer` 或 `X-Inspace-Api-Key`；scope 为 `spaces:read/write`、`guides:read/write`；默认 60 req/min；API Key 绑定用户只能管理自己的对象。
- 新增一次性 Key 工具 `app/src/bin/create-agent-key.rs`；明文只显示一次，数据库不存明文。
- 公网根 `/api/` 已被 HAPI 占用，Nginx 只将精确的 `/api/spaces`、`/api/guides`（含 `/:id`）代理到 3001，其余 `/api/*` 仍去 3006，未破坏 HAPI。
- 完整文档：`docs/AGENT_REST_API.md`。

### 部署与 QA
- `cargo check -p instant-space-app --all-targets` 通过；release server 二进制已安装到 `/usr/local/bin/instant-space-app`；`/health`、`/ready` 正常。
- API 真实临时 Key CRUD 验证：创建/更新空间、创建/更新志、中文多关键词 GET、scope 403、审计均通过；所有临时 Key/空间/志/对应审计均删除，残留 0。
- 浏览器最终证据：`output/playwright/final-20260730/`。
  - 1440×900、390×844：首页当前可见图片正常、无横向溢出。
  - 攻略国家筛选 + `南昌 滕王阁` 返回 1 篇并可点击进入详情。
  - 无 hydration panic、console error、HTTP >=400、横向溢出、未标记表单字段或小于 44px 的主内容交互控件。

## 2026-07-31 — Agent API 补全：详情 + 删除端点

- 用户反馈此前 Agent API 不完整：只有列表/创建/更新，读不回完整正文、删不掉对象。补齐 4 个端点（`app/src/agent_api.rs`，路由加 `get(...).delete(...)`）：
  - `GET /api/spaces/:id`（spaces:read）→ 返回 `SpaceDetail` 完整详情
  - `DELETE /api/spaces/:id`（spaces:write）→ 物理删除，级联清志/聊天/故事/胶囊/认领
  - `GET /api/guides/:id`（guides:read）→ 返回 `GuideDetail`（含 summary/content/sections/images）
  - `DELETE /api/guides/:id`（guides:write）→ 物理删除
- 新增 db 函数 `instant_db::spaces::delete_space`（`crates/db/src/spaces.rs`）；guides 侧复用既有 `delete_guide_row`。外键全部 `ON DELETE CASCADE/SET NULL`，删空间行即可级联。
- 关键陷阱：`GuideSection` 字段名是 `type/title_zh/content_zh/images`，不是 `heading_zh/body_zh`（Agent 或本人传错字段名会静默丢弃 sections）。
- 部署：仅服务端改动，WASM v91 不变、无需重建 WASM。release 二进制已安装重启，`/health ok`、`/ready 200`。
- 端到端验证：建空间 201 → GET 详情 200（含描述/标签）→ 建志 201 → GET 详情 200（sections 原样返回）→ DELETE 志 204 → DELETE 空间 204 → 删除后再 GET 404 → 只读 scope 删除 403。临时 Key/数据全部清理，残留 0。
- 文档已更新：`docs/AGENT_REST_API.md`（详情/删除端点表 + sections 字段说明）。

## 2026-07-31 — Agent API 口语教程文档

- 新增 `docs/AGENT_API_TUTORIAL.md`（284 行，口语化中文教程）：从建 Key → 建空间 → 写攻略（含 sections 正确字段名提醒）→ 读回 → 搜索 → PATCH 更新/发布 → DELETE 清理，附 curl 与 Python 完整示例、错误码速查、运营流程模板。
- `docs/AGENT_REST_API.md` 保留为技术参考（字段全表/错误码），教程面向"怎么用"。

## 2026-07-31 — Phase 1 统一错误反馈模型（feedback 模块）

- 新增 `app/src/feedback.rs`：全局 Feedback 上下文（Success/Error/Info），`use_feedback()` 提供 `success/error/info/dismiss`，消息 4.5s 自动消失；`FeedbackToasts` 全局 toast 轨渲染在 `App` 的 `app-main` 内。
- 依赖：workspace 与 app 增加 `gloo-timers`（仅 hydrate feature）；定时器用 `leptos::task::spawn_local`（不是 `leptos::prelude::spawn_local`）。
- 已迁移：`space_form`（建空间成功、关联攻略成功）、`auth`（登录/注册成功）发统一 toast；表单内联错误保留（就近可读原则：成功走全局、错误就近）。
- CSS：`app-shell.css?v=20260731-shell-feedback-v5` 新增 `.feedback-rail/.feedback-toast`（底部居中、success/error/info 三态、reduced-motion 支持）。
- 检查：`cargo check --all-targets` 与 wasm32 hydrate check 均通过。

## 2026-07-31 — Phase 2 主理人空间管理：操作审计闭环

- 空间管理动作现写入 `admin_audit_log`：close_space / reactivate_space / delete_space（`set_my_space_status` 增加 action 参数）、archive_space_template、regenerate_space_password（含新 password_version）。
- 描述/标签/有效期编辑已存在（`update_my_space` + host 编辑表单），该项视为已闭环。
- 未做（属白皮书 Phase 2 长期项，非本轮工程缺口）：商家/创作者认领流程、`space_templates` 数据闭环。

## 2026-07-31 — Phase 3 私密空间与访问：成员角色最小闭环

- `space_members` 表已有外键，补产品闭环：
  - db（`crates/db/src/spaces.rs`）：`SpaceMember`（含 email/display_name）、`list_space_members`、`set_space_member`（upsert，role 仅 member/host 两档）、`remove_space_member`、`find_user_id_by_email`。
  - server（`app/src/server/spaces.rs`）：`list_my_space_members`、`add_my_space_member`（按邮箱，校验空间管理者）、`remove_my_space_member`，管理动作写 `admin_audit_log`。
  - UI：`ManageSpacePanel` 新增 `SpaceMembersPanel`（按邮箱邀请、角色选择、成员列表、移除；成功/失败走统一 feedback toast）。
- `crates/db/Cargo.toml` 增加 `serde.workspace = true`（SpaceMember derive）。
- 样式：`workspace.css?v=20260731-members-v18` 新增 `.space-members-*`（桌面/手机双布局，44px 控件）。
- 未做（属权限中心长期项）：QR/GPS/Discord 证明强度分级、消息级权限、角色细粒度权限矩阵。

## 2026-07-31 — Phase 4-7 全面实现（工程缺口闭环）

### Phase 4 攻略媒体资产
- 新增迁移 `20260731000300_guide_versions.sql`：`guide_versions` 表（每次创建/编辑自动快照，版本号递增，UNIQUE(guide_id, version_no)）。
- db（`crates/db/src/guides.rs`）：`snapshot_guide_version` / `list_guide_versions` / `restore_guide_version`（恢复前先快照当前态，恢复不丢数据；保留 status/featured/归属）。
- server：创建/更新攻略后自动写快照；新增 `list_guide_versions`、`restore_guide_version`（编辑者或管理员）。
- UI：攻略编辑器新增「版本历史」卡片（列表 + 恢复按钮）；删除攻略改为两步确认。
- 媒体上传：新增 `app/src/media.rs` 裸 Axum 路由 `POST /api/media/upload`（同时注册 `/inspace/api/media/upload`，session cookie 鉴权，mime 限 jpeg/png/webp/gif/avif，大小 ≤10MB，存 `uploads/`），`/uploads` ServeDir 提供访问；`ImageManager` 加「上传图片」按钮（FormData + fetch，需要 web-sys File/FormData/Blob/Url + wasm-bindgen-futures/js-sys）。
- 注意：nginx `/inspace/api` 需 `client_max_body_size 12m` 才能传大图；上传 URL 返回 `/inspace/uploads/<uuid>.<ext>`。

### Phase 5 后台运营
- 攻略后台改服务端分页/搜索/状态筛选（`list_admin_guides_page` + db `list_all_guides_admin_page`，每页 20，含总数）；统计卡片走 `guide_status_counts`。
- CSV 导出：db `export_spaces_csv` / `export_guides_csv`（转义正确）；server `export_admin_csv(kind)`；UI `ExportCsvButton`（Blob 下载，仅管理员）；空间/攻略后台各一个按钮。
- 危险操作确认统一：攻略删除两步确认；空间删除已有两步确认（Phase 2 完成）。

### Phase 6 实时互动
- 迁移 `20260731000400_chat_kind_and_helps.sql`：`chat_messages.kind`（text/system/help/help_resolved）+ `helps.requester_name` / `resolved_at` + 聊天索引。
- domain：`ChatMessageKind`、`ChatMessage.kind`、`SpaceHelp`；db：`insert_message` 带 kind，helps 的 `create_help/resolve_help/list_active_helps`。
- realtime：房间连接上限 300（升级前拒绝）、每连接 400ms 消息限流、WS 消息支持 kind。
- 求助闭环：聊天页「现场求助」表单 + 活跃求助列表 + 「已解决」按钮；发起/解决都会写入聊天室 system/help 消息并广播。
- CSS：`.chat-help-*`、`.chat-message--help/system/help-resolved`（ui-system.css）。

### Phase 7 生产化
- 迁移漂移检测：`instant_db::verify_schema_contract(pool)`（信息模式校验 15 张关键表的表/列契约），启动时调用，漂移打 ERROR 日志（不阻断启动）。
- importer：`--import --pg URL` 真实导入旧 SQLite（用户/空间/攻略动态列映射、稳定 UUIDv5、space_type 归一化、密码占位哈希），目标库已有数据则拒绝（幂等防重）。
- CI：`.github/workflows/ci.yml`（fmt + workspace check + wasm hydrate check + 单测）。
- 备份文档：`docs/BACKUP_AND_RECOVERY.md`（pg_dump + uploads 目录、14 天保留、恢复演练步骤）。

### 编译与部署要点
- `cargo check --workspace --all-targets` 与 wasm32 hydrate check（`--no-default-features --features hydrate`）均通过。
- 类型注意：分页/统计/成员类型必须放 `instant_domain`（`PaginatedGuides`/`GuideStatusCounts`/`SpaceMember`），否则 hydrate 编译报找不到 instant_db。
- WASM 版本：前端与服务端都变了，v91 → v92（`app/src/main.rs` `.output_name` 与 `scripts/build-wasm.mjs` OUTPUT_NAME 同步）。
- 部署顺序：`CARGO_BUILD_JOBS=3 cargo build --release` → `node scripts/build-wasm.mjs` → 安装二进制 → nginx 加 `/inspace/api` 的 client_max_body_size → reload → restart → 日志确认 `schema contract verified`。
