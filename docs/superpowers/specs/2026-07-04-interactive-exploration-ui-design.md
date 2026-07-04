# Instant Space Interactive Exploration UI Design

## Status

Approved direction: build an orderly, interactive exploration interface with a light playful tone.

This is not an Apple-style marketing homepage and not a decorative "fun" page. The product remains a usable map-first application. The UI should feel composed, clickable, and alive, with enough warmth to make discovery feel inviting.

## Source Inputs

- Product: Instant Space, a map-based location and space discovery app.
- Current implementation: Leptos/Axum SSR app with MapLibre, homepage space listing, private password verification, guide/admin shells, and WASM hydration.
- User feedback: current UI has no visual polish, weak interaction affordance, and no coherent aesthetic system.
- Follow-up map direction:
  - Keep MapLibre as the browser WebGL map engine.
  - Use Rust/WASM for the browser-side map control layer, not a large hand-written JavaScript map implementation.
  - Do not use MapLibre's official demo/default style as the real product basemap.
  - Use a free, mature, MapLibre-compatible basemap provider. Approved provider: OpenFreeMap.
  - Support dynamic projection switching between a flat 2D map and a 3D globe.
- UI/UX Pro Max guidance used manually:
  - Product query: playful map discovery, social travel, local exploration, vibrant consumer web app.
  - Recommended style: bento grids and modular discovery cards, adapted for an app interface rather than a marketing landing page.
  - Recommended colors: local discovery orange plus map blue, with enough neutral structure to avoid visual conflict.
  - Required UX checks: clear loading states, no excessive motion, visible focus, touch targets, responsive breakpoints, reduced-motion support.
- Design System guidance used manually:
  - Three-layer token model: primitive -> semantic -> component.
  - Component states: default, hover, focus, active, disabled, loading, error.
  - No ad hoc component hex colors once tokens exist.

## Design Goal

Make the Rust app look intentionally designed while preserving the map-first product behavior.

The homepage should answer three questions immediately:

1. Where am I exploring?
2. What spaces are available?
3. What can I interact with right now?

The design should have visual charm, but the charm must come from rhythm, color discipline, clear affordances, and small state changes. It should not rely on random decorative blobs, oversized marketing hero copy, or noisy animation.

## Non-Goals

- No landing-page-only first screen.
- No full-screen promotional hero before the map.
- No decorative gradient orb background.
- No nested cards inside cards.
- No emoji used as structural icons.
- No large UI library migration in this pass.
- No complete rewrite of unfinished product flows in the visual pass.
- No attempt to make admin pages playful in a way that reduces operational clarity.
- No Leaflet migration or reuse of the old Next.js map stack.
- No production dependency on MapLibre demo tiles or demo style JSON.
- No raw public OSM tile URL as the product basemap.
- No expensive/commercial map provider requirement for the first pass.

## Product Personality

Instant Space should feel like a city exploration instrument:

- Clear enough to use repeatedly.
- Warm enough to invite exploration.
- Lightly playful through microcopy, badges, and motion.
- Trustworthy enough for private spaces, passwords, and admin workflows.

The tone should avoid both extremes:

- Too premium/static: looks polished but feels like a brochure.
- Too playful/noisy: looks fun but fights the map and data.

## Information Architecture

### Global Shell

The global shell should use a stable top navigation on desktop and a compact header on mobile.

Primary navigation:

- Home or Explore
- Guides
- Create Space
- Admin

The current route set can remain. The visual treatment changes first:

- Brand area has a compact mark and "Instant Space".
- Navigation links have clear hover/focus/active states.
- The primary action is "Create Space".
- Admin is visually quieter than user-facing exploration links.

### Homepage

The homepage remains the product's main experience.

Desktop layout:

- Full-viewport map surface.
- Floating top navigation above map.
- Floating search and filter panel over the map.
- Compact space drawer for browsing available spaces.
- Right-side detail drawer only after a space is selected.
- No separate hero section before the map.

Mobile layout:

- Header at top.
- Map remains the first-screen surface.
- Search/filter controls sit above the map as a compact panel.
- Space list becomes a bottom drawer-like surface.
- Search and filters remain reachable without horizontal scrolling.

### Guides, Login, Host, Admin

These pages should receive global tokens and component styling, but their deeper product workflows can remain scoped follow-ups.

Visual pass expectations:

- Remove placeholder feel.
- Use consistent page headers, panels, forms, inputs, and buttons.
- Keep admin denser and calmer than exploration pages.
- Do not invent fake functionality.

## Homepage Components

### Map Runtime Architecture

Purpose: keep the Rust rewrite honest. The map should not become a mostly JavaScript island with a Rust shell around it.

Architecture:

- `instant-map-ui` is the browser-side Rust/WASM map control crate.
- MapLibre GL JS v5.x is the rendering engine for WebGL, vector tiles, camera, projection, and marker overlay support.
- JavaScript is limited to a thin adapter around MapLibre APIs that cannot be called directly from Rust/WASM.
- Leptos components own UI state and call into the WASM map control layer.
- Server functions and PostgreSQL remain the source of truth for space data.

WASM-owned responsibilities:

- Normalize map provider configuration.
- Track active style and projection mode.
- Serialize and sync space points to the map adapter.
- Fit map bounds to visible spaces.
- Focus/fly to the selected space.
- Preserve selected space, center, zoom, and filters across style/projection changes.
- Expose a small API such as `mount`, `set_style`, `set_projection`, `sync_spaces`, `focus_space`, and `fit_spaces`.

JavaScript adapter responsibilities:

- Create and destroy the MapLibre map instance.
- Call MapLibre methods such as `setStyle`, `setProjection`, `fitBounds`, `flyTo`, and marker construction.
- Report recoverable map initialization errors back to the UI.
- Avoid application-specific filtering, selection, or data ownership.

### Basemap Provider

Approved default provider: OpenFreeMap.

Provider reasons:

- Free to use for the first implementation pass.
- Provides MapLibre-compatible style JSON.
- Does not require a commercial API key for local development.
- Works with vector styles, which are more reliable for MapLibre than ad hoc raster tile URLs.

Initial styles:

- Road: `https://tiles.openfreemap.org/styles/liberty`
- Dark: `https://tiles.openfreemap.org/styles/dark`
- The map container background must follow the active style. Road mode uses the OpenFreeMap road background color so transparent WebGL areas do not read as a dark blank map; dark mode uses the app map background.

Provider rules:

- The product must not silently fall back to `https://demotiles.maplibre.org/style.json`.
- If a style URL fails, show an explicit map error state with recovery language.
- Keep provider configuration centralized so a later pass can switch to Protomaps/PMTiles self-hosting or a paid provider without rewriting UI components.
- OpenFreeMap is accepted as the zero-cost provider, but it is not treated as a commercial SLA service.

### Projection Modes

Purpose: support both practical browsing and the exploration feeling requested for the product.

Modes:

- `2D Map`: Mercator projection for normal browsing, searching, filtering, and selecting spaces.
- `3D Globe`: Globe projection for a more exploratory world view.

Behavior:

- A visible segmented control or compact toggle switches between 2D and 3D.
- Switching projection preserves current center, zoom, selected space, active filters, and open detail drawer.
- Marker data remains synced after projection changes.
- If direct projection switching is stable in MapLibre GL JS v5.x for the active style, use `setProjection`.
- If a style/provider combination cannot switch projection safely, the WASM layer may perform a controlled remount while restoring user state.

### Map Surface

Purpose: make location browsing feel primary.

Design:

- Full-bleed map, not inside a decorative card.
- Stable height: desktop fills viewport; mobile uses a defined min-height and leaves room for controls and drawers.
- Loading fallback should be subtle and branded, not plain text.
- Map controls should remain accessible and not be covered by panels.
- Default view should use a real, readable city-scale basemap. It must not look empty, black, duplicated, or like a demo template.
- Space markers should be visible on top of the basemap and should not be hidden behind fixed controls.

States:

- Loading: skeleton or calm loading label inside reserved map area.
- Ready: MapLibre visible with OpenFreeMap basemap and synced space markers.
- Error or missing MapLibre: show an inline fallback with recovery language.
- Provider error: show a clear basemap loading error rather than falling back to demo tiles.

Controls:

- Road style.
- Dark style.
- 2D map projection.
- 3D globe projection.
- Zoom in/out.
- Optional future satellite mode only if a free provider is selected and verified visually.

### Explorer Panel

Purpose: turn the list of spaces into an interactive discovery surface.

Design:

- Floating panel with strong contrast against map.
- Use a single panel shell, not a stack of unrelated boxes.
- Panel header includes a short title and result count. A live status badge is allowed only when it is backed by existing real data.
- Search, filters, and list are grouped by spacing and hierarchy.

Behavior:

- Search updates results without needing Enter.
- Filtering shows selected states.
- Empty state suggests clearing search or trying a different type.
- Loading state reserves list space and avoids layout jumps.

### Search

Purpose: make exploration direct.

Design:

- Visible label or accessible label plus clear placeholder.
- Input height at least 44px.
- Search icon may be added through CSS or inline SVG later, but not emoji.
- Focus ring uses tokenized map blue or primary orange.

States:

- Default
- Focus
- Filled
- Loading
- Empty result

### Filter Chips

Purpose: make type filtering feel quick and tactile.

Design:

- Horizontal wrap layout, no forced horizontal scroll on mobile.
- Selected chip uses primary color and high-contrast text.
- Unselected chip uses neutral surface with visible border.
- Disabled or unavailable chips are visibly muted.

Initial chip set:

- All
- Scenic
- Food
- Park
- Transit
- Event

### Space Cards

Purpose: make every space look like a selectable object with useful preview data.

Design:

- Cards are repeated list items only.
- Radius stays at 8px or less unless the existing design system later changes it.
- Include name, type badge, city/district line when available, online count, and public/private badge.
- Private spaces show a visually distinct but restrained private badge.
- Online count uses tabular numbers to avoid layout jitter.

Interaction:

- Hover: slight elevation and border/color shift.
- Active: pressed feedback without moving surrounding layout.
- Focus: visible focus ring.
- Selected: persistent visual state if selection is implemented.

### Private Verification

Purpose: make private space entry feel secure, not like a raw form.

Design:

- Inline compact verification block attached to the relevant private space.
- Use a clear title, password input, primary "Enter" action, and inline error/success state.
- Do not expose technical wording like "password version" in the final UI unless needed for debugging.

States:

- Idle
- Submitting
- Success
- Error with clear recovery

### Navigation And Buttons

Purpose: make clickable areas obvious.

Rules:

- All buttons and interactive cards use pointer cursor.
- Primary buttons use orange only when they are the main action.
- Secondary buttons use neutral or outline treatment.
- Link-like navigation should still have focus and active states.
- Minimum interactive height: 40px desktop, 44px mobile.

## Visual System

### Primitive Tokens

Use CSS custom properties in `app/style/main.css`.

Core color primitives:

```css
--color-ink-950: #0f172a;
--color-ink-700: #334155;
--color-ink-500: #64748b;
--color-ink-200: #d7dee8;
--color-ink-100: #eef2f7;
--color-paper: #ffffff;
--color-canvas: #f8fafc;
--color-warm-50: #fff7ed;
--color-orange-600: #ea580c;
--color-orange-500: #f97316;
--color-blue-600: #2563eb;
--color-cyan-700: #0891b2;
--color-green-600: #16a34a;
--color-red-600: #dc2626;
```

Spacing primitives:

```css
--space-1: 4px;
--space-2: 8px;
--space-3: 12px;
--space-4: 16px;
--space-5: 20px;
--space-6: 24px;
--space-8: 32px;
--space-10: 40px;
--space-12: 48px;
--space-16: 64px;
```

Shape, shadow, and motion primitives:

```css
--radius-sm: 4px;
--radius-md: 6px;
--radius-lg: 8px;
--shadow-soft: 0 14px 40px rgb(15 23 42 / 0.14);
--shadow-card: 0 8px 24px rgb(15 23 42 / 0.10);
--duration-fast: 150ms;
--duration-normal: 220ms;
--ease-standard: cubic-bezier(0.2, 0.8, 0.2, 1);
```

### Semantic Tokens

```css
--color-bg: var(--color-canvas);
--color-fg: var(--color-ink-950);
--color-muted-fg: var(--color-ink-500);
--color-surface: var(--color-paper);
--color-surface-warm: var(--color-warm-50);
--color-border: var(--color-ink-200);
--color-primary: var(--color-orange-600);
--color-primary-hover: #c2410c;
--color-primary-fg: #ffffff;
--color-accent: var(--color-blue-600);
--color-accent-fg: #ffffff;
--color-discovery: var(--color-cyan-700);
--color-success: var(--color-green-600);
--color-danger: var(--color-red-600);
--color-ring: var(--color-blue-600);
```

### Component Tokens

```css
--panel-bg: rgb(255 255 255 / 0.94);
--panel-border: rgb(15 23 42 / 0.12);
--panel-shadow: var(--shadow-soft);
--card-bg: var(--color-surface);
--card-border: var(--color-border);
--card-shadow: var(--shadow-card);
--button-bg: var(--color-primary);
--button-fg: var(--color-primary-fg);
--button-radius: var(--radius-lg);
--input-bg: var(--color-surface);
--input-border: var(--color-border);
--badge-radius: var(--radius-md);
```

## Typography

Use system fonts first to keep the current stack simple and fast:

```css
font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
```

If a later pass adds web fonts, use:

- Headings: Nunito 800/900 or a local fallback.
- Body: DM Sans or system fallback.

Rules:

- Body text minimum 16px on mobile.
- No viewport-width font scaling.
- Letter spacing stays at `0`.
- Compact panels use compact headings, not hero-scale type.
- Use tabular numbers for counts and badges.

## Motion

Motion should make the interface feel responsive, not busy.

Allowed:

- Button hover color transition.
- Card hover elevation and subtle translate up.
- Filter chip selected transition.
- Panel entrance only if it does not delay usability.
- Skeleton shimmer or pulse for loading, respecting reduced motion.

Not allowed:

- Infinite decorative animation.
- Scroll-jacking.
- Large parallax.
- Animating layout dimensions like width/height for core content.
- More than two noticeable animated elements in one viewport.

Reduced motion:

```css
@media (prefers-reduced-motion: reduce) {
  * {
    animation-duration: 1ms !important;
    transition-duration: 1ms !important;
    scroll-behavior: auto !important;
  }
}
```

## Responsive Behavior

Breakpoints to verify:

- 375px phone
- 768px tablet
- 1024px laptop
- 1440px desktop

Desktop:

- Map fills viewport.
- Explorer panel width around 360-420px.
- Header and panel do not overlap MapLibre controls.

Tablet:

- Panel may remain side-mounted if width allows.
- Search and chips stay readable.

Mobile:

- Avoid horizontal scroll.
- Header compresses without wrapping awkwardly.
- Explorer panel becomes a bottom section or bottom sheet-like surface.
- Touch targets are at least 44px high.
- Text wraps rather than overflowing.

## Accessibility

Required checks:

- Normal text contrast at least 4.5:1.
- UI component boundaries at least 3:1.
- Focus states visible on all interactive elements.
- Form fields have labels or accessible labels.
- Status changes use visible text, not color alone.
- Buttons and cards preserve semantic roles.
- Map fallback has readable text.
- Loading and error states are not blank.

## Page Scope For First Implementation Pass

Implement first:

1. Global design tokens in `app/style/main.css`.
2. Header visual system.
3. Homepage map layout.
4. Explorer panel.
5. Search and filter chip styling.
6. Space cards and status badges.
7. Private verification visual states.
8. Basic styling improvements for Login, Guides, Host, and Admin using the same tokens.

Defer:

- Complete host space creation flow.
- Complete guide city/district/spot interactions.
- Admin CRUD pages.
- Full icon library installation.
- Dark mode.
- Complex page transitions.

## Implementation Notes

- Prefer class changes and CSS tokens over large component rewrites.
- Keep Leptos server function behavior unchanged unless UI state requires a small prop or class.
- Do not restart or modify the old Next.js app.
- Keep the current Rust app port and build scripts unchanged.
- If icons are added in this pass, use inline SVG helpers or a small local set. Do not use emojis.
- Keep MapLibre full-bleed and unframed.
- Use MapLibre GL JS v5.x for the browser map engine.
- Keep map application behavior in Rust/WASM where practical. The JS file should stay a MapLibre adapter, not the application map controller.
- Use OpenFreeMap as the approved free basemap provider.
- Remove any dependency on MapLibre's official default/demo style for real rendering.
- Add provider/projection configuration in one place instead of scattering style URLs through components.

## Verification Plan

Before claiming implementation complete, run:

```powershell
cargo fmt --all --check
cargo check -p instant-space-app
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm run test:wasm
npm run test:browser
```

Manual visual checks after implementation:

- Desktop `1440x900`: map visible, panel clear, cards interactive.
- Mobile `375x812`: no horizontal scroll, text fits, panel usable.
- Road mode: OpenFreeMap road style is visible and readable, with roads, water, and labels.
- Dark mode: OpenFreeMap dark style is visible and readable, not a blank black canvas.
- Projection switch: 2D and 3D modes switch without losing selected space, markers, filter state, or open drawer state.
- Keyboard: tab order reaches header, search, filters, cards, private form.
- Reduced motion: no distracting animation remains.

## Acceptance Criteria

- The homepage no longer looks like default HTML or a placeholder.
- Visual hierarchy is coherent: map, panel, search, filters, list, private entry.
- Color use is disciplined and does not read as a one-note palette.
- Interactive elements clearly communicate hover, focus, active, selected, loading, and error states.
- The UI feels lightly playful through cards, badges, labels, and micro-interactions.
- Existing browser smoke tests continue to pass.
- The map is rendered from OpenFreeMap or another approved provider, not MapLibre demo tiles.
- Rust/WASM owns map control behavior beyond the minimal MapLibre JS adapter.
- Users can switch between flat 2D map and 3D globe projection.
