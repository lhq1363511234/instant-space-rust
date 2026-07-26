import { expect, test } from "@playwright/test";

const SHANGHAI = "上海市";

test("guide and admin shells render", async ({ page }) => {
  await page.goto("/guides");
  await expect(
    page.getByRole("heading", {
      name: /Browse travel guides by destination|按目的地浏览旅行攻略/,
    }),
  ).toBeVisible();
  await expect(page.getByLabel(/Province|省份/)).toContainText(SHANGHAI);
  const bundGuide = page.getByRole("link", {
    name: /The Bund Guide|外滩导览/,
  });
  await expect(bundGuide).toBeVisible();
  await bundGuide.click();
  await expect(page).toHaveURL(
    /\/guides\/20000000-0000-0000-0000-000000000001$/,
  );
  await expect(
    page.getByRole("heading", { name: /The Bund Guide|外滩导览/ }),
  ).toBeVisible();
  await expect(page.getByRole("heading", { name: "到达" })).toBeVisible();
  await expect(page.getByText("从南京东路步行到达。")).toBeVisible();

  await page.goto("/inspace/admin");
  await expect(
    page.getByRole("heading", {
      name: /Admin sign-in required|需要管理员登录/,
    }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: /Go to sign in|去登录/ }),
  ).toBeVisible();
});

test("signed in users can open the structured guide editor", async ({
  page,
}) => {
  await page.goto("/guides/new");
  await expect(
    page.getByRole("heading", { name: /Sign in first|请先登录/ }),
  ).toBeVisible();

  await page.goto("/login");
  await page.getByLabel(/Email|邮箱/).fill("host@example.com");
  await page.getByLabel(/Password|密码/).fill("123456");
  await page
    .getByRole("button", { name: /Sign in|登录/ })
    .last()
    .click();
  await expect(page.locator(".form-success")).toContainText("host@example.com");

  await page.goto("/guides/new");
  await expect(
    page.getByRole("heading", {
      name: /Structured guide editor|结构化攻略编辑器/,
    }),
  ).toBeVisible();
  await expect(
    page.getByLabel(/Guide Chinese title|攻略中文标题/),
  ).toBeVisible();
  await expect(
    page.getByLabel(/Guide section 1 type|攻略第 1 段类型/),
  ).toBeVisible();
  await expect(
    page.getByLabel(/Guide section 1 Chinese title|攻略第 1 段中文标题/),
  ).toBeVisible();
  await expect(page.getByLabel(/Guide image URL|攻略图片 URL/)).toBeVisible();
  await page.getByRole("button", { name: /Add section|新增板块/ }).click();
  await expect(
    page.getByLabel(/Guide section 2 Chinese title|攻略第 2 段中文标题/),
  ).toBeVisible();

  await page.goto("/guides/new?space_id=10000000-0000-0000-0000-000000000001");
  await expect(page.getByLabel(/Guide Chinese title|攻略中文标题/)).toHaveValue(
    "外滩攻略",
  );
  await expect(page.getByLabel(/Guide province|攻略省份/)).toHaveValue(
    SHANGHAI,
  );
  await expect(
    page.getByText(/Linked space|Current linked space|当前关联空间/),
  ).toBeVisible();
});
