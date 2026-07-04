import { expect, test } from "@playwright/test";

test("homepage exposes an interactive exploration surface", async ({ page }) => {
  await page.goto("/");

  await expect(page.locator(".topbar")).toBeVisible();
  await expect(page.getByRole("link", { name: "Create Space" })).toBeVisible();
  await expect(page.locator(".explorer-panel")).toBeVisible();
  await expect(page.locator(".search-control")).toBeVisible();
  await expect(page.locator(".filter-chip")).toHaveCount(6);
  await expect(page.locator(".filter-chip.is-active")).toHaveText("All");
  await expect(page.locator(".space-card").first()).toBeVisible();
  await expect(page.locator(".space-badge").first()).toBeVisible();

  const primary = await page.evaluate(() =>
    getComputedStyle(document.documentElement).getPropertyValue("--color-primary").trim(),
  );
  expect(primary).toBe("#ea580c");
});

test("homepage keeps exploration usable on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto("/");

  await expect(page.locator(".map-layout")).toBeVisible();
  await expect(page.locator(".explorer-panel")).toBeVisible();
  await expect(page.locator(".space-card").first()).toBeVisible();

  const hasHorizontalOverflow = await page.evaluate(
    () => document.documentElement.scrollWidth > document.documentElement.clientWidth,
  );
  expect(hasHorizontalOverflow).toBe(false);
});
