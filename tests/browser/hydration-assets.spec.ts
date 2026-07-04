import { expect, test } from "@playwright/test";

test("hydration assets are served", async ({ request }) => {
  const loader = await request.get("/pkg/instant_space_app.js");
  await expect(loader).toBeOK();
  await expect(await loader.text()).toContain("export function hydrate");

  const wasm = await request.get("/pkg/instant_space_app_bg.wasm");
  await expect(wasm).toBeOK();
  expect(wasm.headers()["content-type"]).toContain("application/wasm");
});
