import { expect, test } from "@playwright/test";

const PRIVATE_TEA_ROOM = "\u79c1\u5bc6\u8336\u5ba4";

test("private space verification entry is reachable", async ({ page }) => {
  await page.goto("/");
  await page.getByLabel("Search spaces").fill(PRIVATE_TEA_ROOM);
  await expect(page.getByRole("button", { name: PRIVATE_TEA_ROOM })).toBeVisible();
  await page.getByRole("button", { name: PRIVATE_TEA_ROOM }).click();
  await expect(page.getByRole("complementary", { name: "Space detail" })).toBeVisible();
  await expect(page.getByLabel("Private space verification")).toBeVisible();
  await page.getByLabel("Private space password").fill("123456");
  await expect(page.getByRole("button", { name: "Enter chat" })).toBeVisible();
});
