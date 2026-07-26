import { expect, test } from "@playwright/test";

test("homepage explains guide-first mutual help without loading a map", async ({
  page,
}) => {
  const requested: string[] = [];
  page.on("request", (request) => requested.push(request.url()));

  await page.goto("/inspace");

  await expect(page.locator(".topbar")).toBeVisible();
  await expect(
    page.getByRole("heading", {
      name: /Read the guide\. Then ask people who are there\.|先看攻略，再问正在这里的人。/,
    }),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", {
      name: /What do people need after they arrive|到达一个地点之后/,
    }),
  ).toBeVisible();
  await expect(
    page.getByText(
      /Guides turn experience into shared knowledge|攻略让经验成为共同知识/,
    ),
  ).toBeVisible();
  await expect(page.locator("#map")).toHaveCount(0);
  expect(
    requested.some(
      (url) => url.includes("maplibre-gl") || url.includes("/ofm/"),
    ),
  ).toBe(false);
});

test("primary navigation exposes map and guides directly", async ({ page }) => {
  await page.goto("/inspace");
  await expect(
    page.getByRole("link", { name: /Space map|空间地图/ }).first(),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Guides|空间攻略/ }).first(),
  ).toBeVisible();

  await page
    .getByRole("link", { name: /Space map|空间地图/ })
    .first()
    .click();
  await expect(page).toHaveURL(/\/inspace\/map$/);
  await expect(page.locator("#map")).toBeVisible();
});

test("homepage can switch between English and Chinese", async ({ page }) => {
  await page.goto("/inspace");
  await page.getByLabel(/Open navigation menu|打开导航菜单/).click();
  await page.getByRole("button", { name: /EN|English/ }).click();
  await expect(
    page.getByRole("heading", {
      name: "Read the guide. Then ask people who are there.",
    }),
  ).toBeVisible();

  await page.getByRole("button", { name: /中文|Chinese/ }).click();
  await expect(
    page.getByRole("heading", { name: "先看攻略，再问正在这里的人。" }),
  ).toBeVisible();
});

test("mobile homepage keeps four top-level entrances visible", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/inspace");

  const nav = page.locator(".mobile-bottom-nav");
  await expect(nav).toBeVisible();
  await expect(nav.locator("a")).toHaveCount(4);
  await expect(nav.getByText(/Home|首页/)).toBeVisible();
  await expect(nav.getByText(/Space map|空间地图/)).toBeVisible();
  await expect(nav.getByText(/Guides|空间攻略/)).toBeVisible();
  await expect(nav.getByText(/My spaces|我的空间/)).toBeVisible();

  const hasHorizontalOverflow = await page.evaluate(
    () =>
      document.documentElement.scrollWidth >
      document.documentElement.clientWidth,
  );
  expect(hasHorizontalOverflow).toBe(false);
});
