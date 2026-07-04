# Homepage UI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Turn the Rust homepage from a placeholder shell into an orderly, interactive exploration interface.

**Architecture:** Keep the existing Leptos/Axum/MapLibre structure. Add browser tests first, then update the homepage components and CSS tokens without changing database behavior.

**Tech Stack:** Rust, Leptos 0.7, Axum, MapLibre GL JS, Playwright, CSS custom properties.

---

## File Structure

- Modify `tests/browser/homepage-ui.spec.ts`: browser-level UI structure and responsive checks.
- Modify `app/src/components/header.rs`: readable global navigation and primary action styling hooks.
- Modify `app/src/components/map_home.rs`: explorer panel, filters, cards, empty/loading states.
- Modify `app/src/components/private_verify.rs`: non-technical private entry states.
- Modify `app/src/server/spaces.rs`: add location preview fields to map marker payload.
- Modify `app/style/main.css`: design tokens, homepage layout, controls, cards, responsive rules.

## Tasks

### Task 1: Add Homepage UI Browser Contract

- [ ] Add Playwright tests that require the explorer panel, filter chips, cards, CSS tokens, and mobile no-overflow behavior.
- [ ] Run `npm run test:browser -- tests/browser/homepage-ui.spec.ts`.
- [ ] Expected before implementation: fail because current homepage lacks the new structure.

### Task 2: Implement Homepage Component Structure

- [ ] Update header, map home, private verification, and space marker data.
- [ ] Keep server functions and route behavior unchanged.
- [ ] Run `cargo fmt --all --check` and `cargo check -p instant-space-app`.

### Task 3: Implement Tokenized CSS

- [ ] Replace placeholder CSS with primitive, semantic, and component tokens.
- [ ] Style map, panel, search, chips, cards, private entry, and basic secondary pages.
- [ ] Add responsive rules for 375px, tablet, and desktop.

### Task 4: Verify

- [ ] Run `npm run test:wasm`.
- [ ] Run `npm run test:browser`.
- [ ] Capture or inspect the local page after changes.
- [ ] Commit after verification.
