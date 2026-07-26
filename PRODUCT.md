# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users
旅行者与本地探索者：已经用地图导航到达（或计划到达）一个真实地点，需要在“到达之后”快速获得该地点的攻略、现场状态与同好讨论。次要用户：空间主理人（创建/运营地点空间）、平台管理员（内容与用户运营）。

## Product Purpose
inspace（SpaceOS）为每一个真实地点提供一个可通过地图、链接或二维码进入的数字空间。地图解决“到达（Arrive）”，inspace 解决“进入与体验（Experience）”。当前 Phase 1 落地为旅行攻略空间：地图找地点 → 进入空间 → 看攻略 → 参与讨论 → 扫码分享。

## Positioning
构建物理世界的介观层（Meso Layer）：鸟眼=地图导航，蛙眼=二维码/NFC，介观=inspace 空间。定位公式：Google Maps + SpaceOS = 完整的数字地球。邻近产品（地图、点评、攻略站）都不以“可进入的地点数字空间”为原语。

## Operating Context
- 手机为主要现场使用设备（到达地点后扫码/搜索进入）；桌面用于浏览、创作攻略与后台运营。
- 线上入口 https://opctoai.com/inspace （路由前缀 /inspace）。
- 空间可公开或凭密码进入；空间内含攻略（Guides）、实时聊天（WebSocket）、二维码分享。

## Capabilities and Constraints
- 技术栈：Rust + Leptos（SSR + WASM hydration）+ Axum + PostgreSQL；MapLibre 地图；样式为手写 CSS（app-shell.css / workspace.css / backoffice.css 为现行设计系统层，ui-system.css / main.css 为遗留层）。
- 已有功能：地图发现、探索空间（分类+分页，后端 limit/offset）、攻略、空间详情+聊天、创建/管理空间、用户工作台（我的空间）、管理员后台（概览/首页编辑器/空间/攻略/常驻/用户）。
- 约束：服务器 4 核 13G，release/WASM 构建昂贵，每轮交付最多构建一次；不得破坏地图/WASM/聊天 WebSocket/认证；不得提交密钥。
- 列表必须支撑 1 万+ 空间与攻略（分类→分页，每页 24，显示总数）。

## Brand Commitments
- 对外名称统一 `inspace`（旧名 Instant Space / InSpaceOS 仅作别名保留于 JSON-LD）。
- Slogan：Be IN the space, beyond the map.
- 首页主标题（用户已拍板）：走到导航的尽头，才是体验的开始。
- 副文案：地图带你到达，inspace 让你真正进入——看它的过去、此刻在场的人、以及属于这里的故事。
- 布局承诺：ChatGPT 式左侧固定导航（260↔72px 折叠）+ 右侧内容；移动端抽屉 + 底部 5 Tab；用户工作台与管理员后台入口固定在全局侧栏底部；后台内部仅用紧凑二级导航，禁止侧栏套侧栏。

## Evidence on Hand
- `docs/PRODUCT_VISION.md`、`SpaceOS 白皮书 V2.0.docx`、`InSpaceOS 创始人寄语.docx`（均在仓库根/docs）。
- 真实种子数据：少量空间（外滩、私密茶室等）与攻略；无真实用户评价/媒体报道，不得虚构。

## Product Principles
1. 到达不是终点：每个界面都要回答“进入之后能做什么”。
2. 地点即对象：空间是唯一可卡片化的一等对象；其余内容用排版与分组表达，拒绝框套框。
3. 规模化优先：任何列表都假设 1 万条，先分类/搜索再分页。
4. 现场可用：移动端按现场使用重新构图，触控 ≥44px，正文对比度 ≥4.5:1。
5. 克制的世界感：深色仅用于承载“空间/地图”意象的核心视觉，其余保持明亮、编辑化、留白充分。

## Accessibility & Inclusion
中英双语（中为主）；WCAG AA 对比度；支持 prefers-reduced-motion；键盘可达、焦点可见。
