import { expect, test } from "@playwright/test";

test("unified sign in accepts seeded host account", async ({ page }) => {
  await page.goto("/login");

  await expect(
    page.getByRole("heading", {
      name: /Unified Sign in \/ Register|登录后创建和管理旅行空间/,
    }),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: /Sign in|登录/ }).first(),
  ).toHaveAttribute("aria-pressed", "true");

  await page.getByLabel(/Email|邮箱/).fill("host@example.com");
  await page.getByLabel(/Password|密码/).fill("123456");
  await page
    .getByRole("button", { name: /Sign in|登录/ })
    .last()
    .click();
  await expect(page.locator(".form-success")).toContainText("host@example.com");

  await page.goto("/my-spaces");
  await expect(
    page.getByRole("heading", {
      name: /My Spaces|My travel spaces|我的旅行空间/,
    }),
  ).toBeVisible();
  await page.getByLabel(/Open navigation menu|打开导航菜单/).click();
  await expect(page.getByLabel(/My account|我的账户/)).toBeVisible();
  await page.getByLabel(/My account|我的账户/).click();
  await page.getByRole("button", { name: /Sign out|退出登录/ }).click();
  await expect(
    page.getByRole("link", { name: /Sign in|登录/, exact: true }),
  ).toBeVisible();
});

test("register mode is reachable from the same form", async ({ page }) => {
  await page.goto("/login");

  await page.getByRole("button", { name: /Register|注册/ }).click();
  await expect(page.getByLabel(/Display name|显示名称/)).toBeVisible();
  await expect(
    page.getByRole("button", { name: /Create account|创建账户/ }),
  ).toBeVisible();
});

test("create space opens as modal with map picker", async ({ page }) => {
  test.setTimeout(90_000);
  await page.goto("/login");
  await page.getByRole("button", { name: /Register|注册/ }).click();
  const testEmail = `space-host-${Date.now()}@example.com`;
  await page.getByLabel(/Display name|显示名称/).fill("Space Host");
  await page.getByLabel(/Email|邮箱/).fill(testEmail);
  await page.getByLabel(/Password|密码/).fill("123456");
  await page.getByRole("button", { name: /Create account|创建账户/ }).click();
  await expect(page.locator(".form-success")).toContainText(testEmail);

  await page.goto("/");
  await page.getByLabel(/Open navigation menu|打开导航菜单/).click();
  await page.getByRole("button", { name: "EN", exact: true }).click();
  await page
    .locator(".nav-menu-panel")
    .getByRole("button", { name: /Create Space|创建空间/ })
    .click();
  await expect(
    page.getByRole("dialog", { name: /Create Space|创建空间/ }),
  ).toBeVisible();
  await expect(page.getByLabel("Pick space location map")).toBeVisible();
  await expect(
    page.locator("#create-space-map .maplibregl-ctrl-zoom-in"),
  ).toBeVisible();
  await expect(page.getByLabel("Space password")).toHaveCount(0);

  const latBefore = await page.getByLabel("Latitude").inputValue();
  const lngBefore = await page.getByLabel("Longitude").inputValue();
  await page.locator("#create-space-map .maplibregl-ctrl-zoom-in").click();
  await page
    .getByLabel("Pick space location map")
    .click({ position: { x: 280, y: 160 } });

  await expect
    .poll(async () => page.getByLabel("Latitude").inputValue())
    .not.toBe(latBefore);
  await expect
    .poll(async () => page.getByLabel("Longitude").inputValue())
    .not.toBe(lngBefore);

  const name = `Phase Space ${Date.now()}`;
  await page.getByLabel("Chinese name").fill(name);
  await page.getByLabel("Country").fill("中国");
  await page.getByLabel("Province").fill("上海市");
  await page.getByLabel("City").fill("上海市");
  await page.getByRole("button", { name: "Create", exact: true }).click();
  await expect(page.locator(".form-success")).toContainText(name);
  await expect(page.locator(".password-result")).toContainText("InstantSpace_");
  await page.getByRole("button", { name: "Close", exact: true }).click();

  await page.goto("/my-spaces");
  await expect(
    page.getByRole("heading", {
      name: /My Spaces|My travel spaces|我的旅行空间/,
    }),
  ).toBeVisible();
  let card = page.locator(".my-space-card").filter({ hasText: name });
  await expect(card).toBeVisible();
  await expect(page.getByText("active").first()).toBeVisible();

  await card.getByText("Manage space").click();
  let dialog = page.getByRole("dialog", { name: /Manage space/ });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByText("The old password is encrypted")).toBeVisible();
  await expect(page.getByLabel("Manage space location map")).toBeVisible();
  const manageLatBefore = await dialog
    .getByLabel("Manage latitude")
    .inputValue();
  await page
    .getByLabel("Manage space location map")
    .click({ position: { x: 220, y: 150 } });
  await expect
    .poll(async () => dialog.getByLabel("Manage latitude").inputValue())
    .not.toBe(manageLatBefore);

  const renamed = `${name} Managed`;
  await dialog.getByLabel("Manage Chinese name").fill(renamed);
  await dialog.getByRole("button", { name: "Save changes" }).click();
  await expect(dialog.locator(".form-success")).toContainText("Space updated");
  await dialog.getByRole("button", { name: "Close manage space" }).click();
  await expect(page.getByText(renamed)).toBeVisible();

  card = page.locator(".my-space-card").filter({ hasText: renamed });
  await card.getByText("Manage space").click();
  dialog = page.getByRole("dialog", { name: /Manage space/ });
  await dialog
    .getByRole("button", { name: "Reset and show new password" })
    .click();
  await expect(dialog.locator(".password-result")).toContainText(
    "InstantSpace_",
  );

  await dialog.getByRole("button", { name: "Close space" }).click();
  await expect(dialog.locator(".form-success")).toContainText("Space closed");
  await dialog.getByRole("button", { name: "Close manage space" }).click();
  card = page.locator(".my-space-card").filter({ hasText: renamed });
  await expect(card.getByText("closed")).toBeVisible();

  await card.getByText("Manage space").click();
  dialog = page.getByRole("dialog", { name: /Manage space/ });
  await dialog.getByRole("button", { name: "Reactivate" }).click();
  await expect(dialog.locator(".form-success")).toContainText(
    "Space reactivated",
  );
  await dialog.getByRole("button", { name: "Close manage space" }).click();
  card = page.locator(".my-space-card").filter({ hasText: renamed });
  await expect(card.getByText("active")).toBeVisible();

  await card.getByText("Manage space").click();
  dialog = page.getByRole("dialog", { name: /Manage space/ });
  await dialog.getByRole("button", { name: "Apply resident" }).click();
  await expect(dialog.locator(".form-success")).toContainText(
    "Resident application submitted",
  );

  await dialog.getByRole("button", { name: "Archive template" }).click();
  await expect(dialog.locator(".form-success")).toContainText(
    "Space archived as template",
  );
  await dialog.getByRole("button", { name: "Close manage space" }).click();
  card = page.locator(".my-space-card").filter({ hasText: renamed });
  await expect(
    card.locator(".space-badge", { hasText: "template" }),
  ).toBeVisible();

  await card.getByText("Manage space").click();
  dialog = page.getByRole("dialog", { name: /Manage space/ });
  await dialog.getByRole("button", { name: "Delete space" }).click();
  await dialog.getByRole("button", { name: "Confirm delete" }).click();
  await expect(card).toHaveCount(0);
});
