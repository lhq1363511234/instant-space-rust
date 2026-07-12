import { expect, test } from "@playwright/test";

test("homepage exposes an interactive exploration surface", async ({ page }) => {
  await page.goto("/");

  await expect(page.locator(".topbar")).toBeVisible();
  await expect(page.getByLabel("Open navigation menu")).toBeVisible();
  await page.getByLabel("Open navigation menu").click();
  await expect(page.getByRole("button", { name: "Create Space" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Guides" })).toBeVisible();
  await expect(page.getByRole("link", { name: "My Spaces" })).toBeVisible();
  await expect(page.getByLabel("Primary").getByRole("link", { name: "Admin" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Sign in" })).toBeVisible();
  await page.getByLabel("Open navigation menu").click();
  await expect(page.locator(".map-filter-panel")).toHaveCount(0);
  await page.getByRole("link", { name: "Explore" }).click();
  await expect(page.locator(".map-filter-panel")).toBeVisible();
  await expect(page.locator(".search-control")).toBeVisible();
  await expect(page.locator(".filter-chip")).toHaveCount(6);
  await expect(page.locator(".filter-chip.is-active")).toHaveText("All");
  await expect(page.getByLabel("Explore space results")).toBeVisible();
  await expect(page.getByLabel("Explore space results").locator(".space-list")).toBeVisible();
  await expect(page.locator(".map-controls")).toHaveCount(0);
  await page.getByLabel("Open navigation menu").click();
  await expect(page.locator(".map-style-switcher")).toBeVisible();
  await expect(page.getByRole("button", { name: "Switch map to roadmap" })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  await expect(page.getByRole("button", { name: "Switch map to dark" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Switch map to satellite" })).toHaveCount(0);
  await expect(page.locator(".map-projection-switcher")).toBeVisible();
  await expect(page.getByRole("button", { name: "Switch to 2D map" })).toHaveAttribute(
    "aria-pressed",
    "true",
  );
  await expect(page.locator(".map-zoom-controls")).toBeVisible();
  await expect(page.locator(".space-drawer")).toHaveCount(0);
  await expect(page.locator(".space-detail-drawer")).toHaveCount(0);

  await page.getByLabel("Open navigation menu").click();
  await page.getByLabel("Search spaces").fill("外滩");
  await page.locator(".map-marker").filter({ hasText: "外滩" }).click();
  await expect(page.locator(".space-detail-drawer")).toBeVisible();
  await expect(page.locator(".space-detail-drawer .drawer-close")).toBeVisible();

  const primary = await page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue("--color-primary").trim(),
  );
  expect(primary).toBe("#7c3aed");
});

test("explorer panel opens only when requested", async ({ page }) => {
  await page.goto("/");

  await expect(page.locator(".map-filter-panel")).toHaveCount(0);
  await page.getByRole("link", { name: "Explore" }).click();
  await expect(page.locator(".map-filter-panel")).toBeVisible();
  await page.getByRole("button", { name: "Close explorer panel" }).click();
  await expect(page.locator(".map-filter-panel")).toHaveCount(0);
});

test("map remounts after navigating away and back home", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator("#map")).toHaveAttribute("data-map-style", "roadmap");

  await page.getByLabel("Open navigation menu").click();
  await page.getByRole("link", { name: "Guides" }).click();
  await expect(page.getByRole("heading", { name: "Global Guides" })).toBeVisible();
  await page.getByRole("link", { name: "Explore" }).click();

  await expect(page.locator("#map")).toBeVisible();
  await expect(page.locator("#map")).toHaveAttribute("data-map-style", "roadmap");
});

test("homepage can switch between English and Chinese", async ({ page }) => {
  await page.goto("/");

  await page.getByRole("link", { name: "Explore" }).click();
  await expect(page.getByRole("heading", { name: "Explore live spaces" })).toBeVisible();
  await page.getByLabel("Open navigation menu").click();
  await page.getByRole("button", { name: "中文" }).click();
  await expect(page.getByRole("heading", { name: "探索实时空间" })).toBeVisible();
  await expect(page.getByRole("button", { name: "创建空间" })).toBeVisible();

  await page.getByRole("button", { name: "EN" }).click();
  await expect(page.getByRole("heading", { name: "Explore live spaces" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Create Space" })).toBeVisible();
});

test("homepage keeps exploration usable on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto("/");

  const deviceFlags = await page.evaluate(() => ({
    device: document.documentElement.dataset.device,
    screen: document.documentElement.dataset.screen,
    nav: document.documentElement.dataset.nav,
  }));
  expect(deviceFlags).toEqual({});
  await expect(page.locator(".brand")).toBeVisible();
  await expect(page.getByLabel("Open navigation menu")).toBeVisible();
  await expect(page.getByRole("link", { name: "Explore" })).toBeVisible();
  await expect(page.getByRole("link", { name: "My Spaces" })).toBeVisible();
  await expect(page.locator(".map-controls")).toHaveCount(0);
  await page.getByLabel("Open navigation menu").click();
  await expect(page.getByRole("button", { name: "中文" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Sign in" })).toBeVisible();
  await expect(page.locator(".map-style-switcher")).toBeVisible();
  await expect(page.locator(".map-projection-switcher")).toBeVisible();
  await page.getByLabel("Open navigation menu").click();
  await expect(page.locator(".map-filter-panel")).toHaveCount(0);
  await page.getByRole("link", { name: "Explore" }).click();
  await expect(page.getByRole("heading", { name: "Explore live spaces" })).toBeVisible();
  await expect(page.locator(".map-layout")).toBeVisible();
  await expect(page.locator(".map-filter-panel")).toBeVisible();
  await expect(page.locator(".search-control input")).toBeVisible();
  await expect(page.locator(".filter-chip")).toHaveCount(6);
  await expect(page.locator(".panel-heading-actions")).toBeVisible();
  await expect(page.getByLabel("Explore space results")).toBeVisible();
  await expect(page.getByLabel("Primary")).toHaveCSS("position", "static");
  await expect(page.locator(".space-drawer")).toHaveCount(0);

  const menuBox = await page.getByLabel("Open navigation menu").boundingBox();
  const spacesBox = await page.getByRole("link", { name: "My Spaces" }).boundingBox();
  expect(menuBox?.x ?? 0).toBeGreaterThan(spacesBox?.x ?? 0);

  const hasHorizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > document.documentElement.clientWidth,
  );
  expect(hasHorizontalOverflow).toBe(false);
});

test("map shim uses realistic map styles instead of demo tiles", async ({ page }) => {
  await page.goto("/");

  await expect(page.locator("#map")).toHaveAttribute("data-map-style", "roadmap");
  const mountedStyleKey = await page.locator("#map").getAttribute("data-map-style");
  expect(mountedStyleKey).toBe("roadmap");
  await expect(page.locator("#map")).toHaveAttribute("data-map-projection", "2d");
  await expect(page.locator("#map")).toHaveCSS("background-color", "rgb(248, 244, 240)");

  await page.getByLabel("Open navigation menu").click();
  await page.getByRole("button", { name: "Switch map to dark" }).click();
  await expect(page.locator("#map")).toHaveAttribute("data-map-style", "dark");
  await expect(page.locator("#map")).toHaveCSS("background-color", "rgb(2, 6, 23)");

  await page.getByRole("button", { name: "Switch to 3D globe" }).click();
  await expect(page.locator("#map")).toHaveAttribute("data-map-projection", "3d");
  await page.getByRole("button", { name: "Switch to 2D map" }).click();
  await expect(page.locator("#map")).toHaveAttribute("data-map-projection", "2d");
});
