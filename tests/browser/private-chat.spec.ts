import { expect, test } from "@playwright/test";

test("private space verification entry is reachable", async ({ page }) => {
  await page.goto("/inspace/spaces/10000000-0000-0000-0000-000000000002");
  await expect(
    page.getByRole("heading", {
      name: /Private space requires an access code|私密空间需要访问码/,
    }),
  ).toBeVisible();
  await expect(
    page.getByLabel(/Private space password|私密空间密码/),
  ).toBeVisible();
  await page.getByLabel(/Private space password|私密空间密码/).fill("123456");
  await page.getByRole("button", { name: /Enter chat|进入聊天/ }).click();
  await expect(
    page.getByRole("region", { name: /Space discussion|空间讨论/ }),
  ).toBeVisible();

  const message = `Guide tip ${Date.now()}`;
  await page
    .getByRole("textbox", { name: /Chat message|聊天消息/ })
    .fill(message);
  await page.getByRole("button", { name: /Send|发送/ }).click();
  await expect(page.getByText(message)).toBeVisible();

  await page.reload();
  await expect(
    page.getByRole("region", { name: /Space discussion|空间讨论/ }),
  ).toBeVisible();
});
