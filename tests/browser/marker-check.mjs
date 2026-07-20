// One-shot verification: China filter shows spaces + marker click opens drawer.
// node tests/browser/marker-check.mjs
import { chromium } from "playwright";

const base = process.env.BASE_URL || "http://127.0.0.1:3001";
const browser = await chromium.launch();
const page = await browser.newContext({ viewport: { width: 1280, height: 900 } }).then((c) => c.newPage());

const errors = [];
page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()); });

await page.goto(`${base}/inspace?country=China`, { waitUntil: "networkidle" }).catch(() => {});
// Give hydration + spaces resource time to resolve and render markers/proxies.
await page.waitForTimeout(4000);

const markerCount = await page.locator(".map-marker").count();
const proxyCount = await page.locator("[data-space-open]").count();

let drawerOpened = false;
if (markerCount > 0) {
  await page.locator(".map-marker").first().click({ force: true }).catch(() => {});
  await page.waitForTimeout(1200);
  drawerOpened = (await page.locator(".space-detail-drawer").count()) > 0;
}

console.log(JSON.stringify({
  markerCount,
  proxyCount,
  drawerOpened,
  consoleErrors: errors.slice(0, 5),
}, null, 2));

await browser.close();
