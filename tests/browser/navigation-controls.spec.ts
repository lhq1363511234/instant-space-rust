import { expect, test } from "@playwright/test";

test("navigation controls hydrate and remain operable", async ({ page }) => {
  const pageErrors: string[] = [];
  page.on("pageerror", (error) => pageErrors.push(error.message));

  await page.goto("/inspace");
  await page.getByLabel(/Open navigation menu|打开导航菜单/).click();
  await expect(page.locator("details.nav-menu")).toHaveAttribute("open", "");

  await page
    .locator(".nav-menu-panel")
    .getByRole("button", { name: /Create Space|创建空间/ })
    .click();
  await expect(
    page.getByRole("dialog", { name: /Create Space|创建空间/ }),
  ).toBeVisible();
  const createDialog = page.getByRole("dialog", {
    name: /Create Space|创建空间/,
  });
  await createDialog.getByRole("button", { name: /Close|关闭/ }).click();
  await expect(createDialog).toHaveCount(0);

  await expect(page.locator("details.nav-menu")).toHaveAttribute("open", "");
  await page.getByRole("button", { name: /EN|English/ }).click();
  await expect(
    page
      .getByRole("link", { name: /Open space map|打开空间地图|空间地图/ })
      .first(),
  ).toBeVisible();

  expect(pageErrors).toEqual([]);
});

test("page-first routes do not load MapLibre before map workspace", async ({
  page,
}) => {
  const requested: string[] = [];
  page.on("request", (request) => requested.push(request.url()));

  await page.goto("/inspace/explore");
  await expect(
    page.getByRole("heading", {
      name: /Find the Space you need|找到你要去的空间/,
    }),
  ).toBeVisible();
  expect(
    requested.some(
      (url) => url.includes("maplibre-gl") || url.includes("/ofm/"),
    ),
  ).toBe(false);
});
