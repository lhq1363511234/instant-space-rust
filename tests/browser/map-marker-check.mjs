import { chromium } from "playwright";
const base = process.env.BASE_URL || "http://127.0.0.1:3001";
const browser = await chromium.launch();
const page = await browser.newContext({ viewport: { width: 1360, height: 900 } }).then((c) => c.newPage());
const errors = [];
page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()); });
await page.goto(`${base}/inspace/map`, { waitUntil: "networkidle" }).catch(() => {});
await page.waitForTimeout(6000);
const markerCount = await page.locator(".map-marker").count();
const libreMarkers = await page.locator("#map .maplibregl-marker").count();
const proxyCount = await page.locator("[data-space-select]").count();
let drawerOpened = false;
if (markerCount > 0) {
  await page.locator(".map-marker").first().click({ force: true }).catch(() => {});
  await page.waitForTimeout(1500);
  drawerOpened = (await page.locator(".space-detail-drawer").count()) > 0;
}
await page.screenshot({ path: "/tmp/map-markers.png" });
console.log(JSON.stringify({ markerCount, libreMarkers, proxyCount, drawerOpened, errors: errors.slice(0,5) }, null, 2));
await browser.close();
