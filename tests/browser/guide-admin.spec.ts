import { expect, test } from "@playwright/test";

const GUIDES = "\u5bfc\u89c8";
const ADMIN = "\u7ba1\u7406\u540e\u53f0";
const SHANGHAI = "\u4e0a\u6d77\u5e02";

test("guide and admin shells render", async ({ page }) => {
  await page.goto("/guides");
  await expect(page.getByRole("heading", { name: GUIDES })).toBeVisible();
  await expect(page.getByLabel("Province")).toContainText(SHANGHAI);

  await page.goto("/admin");
  await expect(page.getByRole("heading", { name: ADMIN })).toBeVisible();
  await expect(page.getByRole("link", { name: "Spaces" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Guides" })).toBeVisible();
});
