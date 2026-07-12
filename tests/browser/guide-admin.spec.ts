import { expect, test } from "@playwright/test";

const GUIDES = "Global Guides";
const ADMIN_REQUIRED = "Admin sign-in required";
const SHANGHAI = "\u4e0a\u6d77\u5e02";

test("guide and admin shells render", async ({ page }) => {
  await page.goto("/guides");
  await expect(page.getByRole("heading", { name: GUIDES })).toBeVisible();
  await expect(page.getByLabel("Province")).toContainText(SHANGHAI);
  await expect(page.getByRole("link", { name: "The Bund Guide" })).toBeVisible();
  await page.getByRole("link", { name: "The Bund Guide" }).click();
  await expect(page).toHaveURL(/\/guides\/20000000-0000-0000-0000-000000000001$/);
  await expect(page.getByRole("heading", { name: "The Bund Guide" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Guide sections" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "到达" })).toBeVisible();
  await expect(page.getByText("从南京东路步行到达。")).toBeVisible();

  await page.goto("/inspace/admin");
  await expect(page.getByRole("heading", { name: ADMIN_REQUIRED })).toBeVisible();
  await expect(page.getByRole("link", { name: "Go to sign in" })).toBeVisible();
});

test("signed in users can open the structured guide editor", async ({ page }) => {
  await page.goto("/guides/new");
  await expect(page.getByRole("heading", { name: "Sign in first" })).toBeVisible();

  await page.goto("/login");
  await page.getByLabel("Email").fill("host@example.com");
  await page.getByLabel("Password").fill("123456");
  await page.getByRole("button", { name: "Sign in" }).last().click();
  await expect(page.locator(".form-success")).toContainText("host@example.com");

  await page.goto("/guides/new");
  await expect(page.getByRole("heading", { name: "Structured guide editor" })).toBeVisible();
  await expect(page.getByLabel("Guide Chinese title")).toBeVisible();
  await expect(page.getByLabel("Guide section 1 type")).toBeVisible();
  await expect(page.getByLabel("Guide section 1 Chinese title")).toBeVisible();
  await expect(page.getByLabel("Guide image URL")).toBeVisible();
  await page.getByRole("button", { name: "Add section" }).click();
  await expect(page.getByLabel("Guide section 2 Chinese title")).toBeVisible();

  await page.goto("/guides/new?space_id=10000000-0000-0000-0000-000000000001");
  await expect(page.getByLabel("Guide Chinese title")).toHaveValue("外滩攻略");
  await expect(page.getByLabel("Guide province")).toHaveValue(SHANGHAI);
  await expect(page.getByText("Linked space:")).toBeVisible();
});
