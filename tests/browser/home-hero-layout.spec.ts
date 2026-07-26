import { expect, test } from "@playwright/test";

test("desktop homepage keeps copy and place example in one composition", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto("/inspace");

  const hero = page.locator(".inspace-home-hero");
  const copy = hero.locator(".inspace-home-hero-copy");
  const visual = hero.locator(".home-space-preview");
  await expect(hero).toBeVisible();
  await expect(visual).toBeVisible();

  const [copyBox, visualBox] = await Promise.all([
    copy.boundingBox(),
    visual.boundingBox(),
  ]);
  expect(copyBox).not.toBeNull();
  expect(visualBox).not.toBeNull();
  expect(Math.abs((copyBox?.y ?? 0) - (visualBox?.y ?? 0))).toBeLessThan(150);
  expect((visualBox?.x ?? 0) + (visualBox?.width ?? 0)).toBeLessThanOrEqual(
    1440,
  );
});

test("mobile homepage has touch-sized fixed navigation and no overflow", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/inspace");

  await expect(page.locator(".brand span:not(.brand-mark)")).toHaveCSS(
    "color",
    "rgb(8, 35, 61)",
  );

  const firstItem = page.locator(".mobile-bottom-nav a").first();
  const itemBox = await firstItem.boundingBox();
  expect(itemBox?.height ?? 0).toBeGreaterThanOrEqual(44);

  const hasHorizontalOverflow = await page.evaluate(
    () =>
      document.documentElement.scrollWidth >
      document.documentElement.clientWidth,
  );
  expect(hasHorizontalOverflow).toBe(false);
});
