import { expect, test } from "@playwright/test";

const PRIVATE_TEA_ROOM = "\u79c1\u5bc6\u8336\u5ba4";
const PRIVATE_TEA_ROOM_EN = "Private Tea Room";

test("private space verification entry is reachable", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("link", { name: "Explore" }).click();
  await page.getByLabel("Search spaces").fill(PRIVATE_TEA_ROOM);
  const marker = page.locator(".map-marker").filter({ hasText: PRIVATE_TEA_ROOM });
  await expect(marker).toBeVisible();
  await marker.click();
  await expect(page.getByRole("complementary", { name: "Space detail" })).toBeVisible();
  await expect(page.getByLabel("Private space verification")).toBeVisible();
  await page.getByLabel("Private space password").fill("123456");
  await expect(page.getByRole("button", { name: "Enter chat" })).toBeVisible();
  await page.getByRole("button", { name: "Enter chat" }).click();
  await expect(page.getByRole("link", { name: "Open private chat" })).toBeVisible();
  await page.getByRole("link", { name: "Open private chat" }).click();
  await expect(page.getByRole("region", { name: "Space chat" })).toBeVisible();

  const message = `Guide tip ${Date.now()}`;
  await page.getByRole("textbox", { name: "Chat message" }).fill(message);
  await page.getByRole("button", { name: "Send" }).click();
  await expect(page.getByText(message)).toBeVisible();

  await page.reload();
  await expect(page.getByRole("region", { name: "Space chat" })).toBeVisible();
});
