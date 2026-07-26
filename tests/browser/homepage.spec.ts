import { expect, test } from "@playwright/test";

test("homepage renders the approved InSpaceOS value proposition", async ({
  page,
}) => {
  await page.goto("/inspace");
  await expect(page).toHaveTitle(/InSpaceOS/);
  await expect(
    page.getByRole("heading", {
      name: /Read the guide\. Then ask people who are there\.|先看攻略，再问正在这里的人。/,
    }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Find a Space|查找一个空间/ }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Browse guides|浏览空间攻略/ }),
  ).toBeVisible();
});
