import { expect, test } from "@playwright/test";

const BUND = "\u5916\u6ee9";
const BUND_EN = "The Bund";

test("homepage renders map and seeded spaces", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByLabel("Instant Space map")).toBeVisible();
  await page.getByRole("link", { name: "Explore" }).click();
  await page.getByLabel("Search spaces").fill(BUND);
  await expect(page.locator(".map-marker").filter({ hasText: BUND })).toBeVisible();
});
