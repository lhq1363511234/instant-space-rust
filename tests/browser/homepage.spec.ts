import { expect, test } from "@playwright/test";

const BUND = "\u5916\u6ee9";

test("homepage renders map and seeded spaces", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByLabel("Instant Space map")).toBeVisible();
  await expect(page.getByRole("button", { name: BUND })).toBeVisible();
  await page.getByLabel("Search spaces").fill(BUND);
  await expect(page.getByRole("button", { name: BUND })).toBeVisible();
});
