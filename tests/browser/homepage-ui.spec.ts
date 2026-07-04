import { expect, test } from "@playwright/test";

test("homepage exposes an interactive exploration surface", async ({ page }) => {
  await page.goto("/");

  await expect(page.locator(".topbar")).toBeVisible();
  await expect(page.getByRole("link", { name: "Create Space" })).toBeVisible();
  await expect(page.locator(".map-filter-panel")).toBeVisible();
  await expect(page.locator(".search-control")).toBeVisible();
  await expect(page.locator(".filter-chip")).toHaveCount(6);
  await expect(page.locator(".filter-chip.is-active")).toHaveText("All");
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
  await expect(page.locator(".space-drawer")).toBeVisible();
  await expect(page.locator(".space-card").first()).toBeVisible();
  await expect(page.locator(".space-badge").first()).toBeVisible();
  await expect(page.locator(".space-detail-drawer")).toHaveCount(0);

  await page.locator(".space-card-action").first().click();
  await expect(page.locator(".space-detail-drawer")).toBeVisible();
  await expect(page.locator(".space-detail-drawer .drawer-close")).toBeVisible();

  const primary = await page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue("--color-primary").trim(),
  );
  expect(primary).toBe("#7c3aed");
});

test("homepage keeps exploration usable on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto("/");

  await expect(page.locator(".map-layout")).toBeVisible();
  await expect(page.locator(".map-filter-panel")).toBeVisible();
  await expect(page.locator(".space-drawer")).toBeVisible();
  await expect(page.locator(".space-card").first()).toBeVisible();
  await expect(page.locator(".map-style-switcher")).toBeVisible();
  await expect(page.locator(".map-projection-switcher")).toBeVisible();

  const hasHorizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > document.documentElement.clientWidth,
  );
  expect(hasHorizontalOverflow).toBe(false);
});

test("map shim uses realistic map styles instead of demo tiles", async ({ page }) => {
  await page.goto("/");

  const mountedStyleKey = await page.locator("#map").getAttribute("data-map-style");
  expect(mountedStyleKey).toBe("roadmap");
  await expect(page.locator("#map")).toHaveAttribute("data-map-projection", "2d");
  await expect(page.locator("#map")).toHaveCSS("background-color", "rgb(248, 244, 240)");

  await page.getByRole("button", { name: "Switch map to dark" }).click();
  await expect(page.locator("#map")).toHaveAttribute("data-map-style", "dark");
  await expect(page.locator("#map")).toHaveCSS("background-color", "rgb(2, 6, 23)");

  await page.getByRole("button", { name: "Switch to 3D globe" }).click();
  await expect(page.locator("#map")).toHaveAttribute("data-map-projection", "3d");
  await page.getByRole("button", { name: "Switch to 2D map" }).click();
  await expect(page.locator("#map")).toHaveAttribute("data-map-projection", "2d");
});
