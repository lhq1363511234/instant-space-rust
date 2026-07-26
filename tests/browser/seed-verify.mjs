import { chromium } from "playwright";
const base = process.env.BASE_URL || "https://opctoai.com";
const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 1360, height: 900 } });
const page = await ctx.newPage();
const errors = [];
page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()); });

// --- map: clustering ---
await page.goto(`${base}/inspace/map`, { waitUntil: "networkidle" }).catch(() => {});
await page.waitForTimeout(7000);
const worldPins = await page.locator(".map-marker").count();
const worldClusters = await page.locator(".map-cluster").count();
await page.screenshot({ path: "/tmp/v-map-world.png" });

// click biggest cluster to zoom in
let afterZoom = null;
if (worldClusters > 0) {
  await page.locator(".map-cluster").first().click({ force: true }).catch(() => {});
  await page.waitForTimeout(2500);
  afterZoom = { pins: await page.locator(".map-marker").count(), clusters: await page.locator(".map-cluster").count() };
  await page.screenshot({ path: "/tmp/v-map-zoom.png" });
}

// open a pin -> drawer
let drawerOpened = false;
if (await page.locator(".map-marker").count() > 0) {
  await page.locator(".map-marker").first().click({ force: true }).catch(() => {});
  await page.waitForTimeout(1800);
  drawerOpened = (await page.locator(".space-detail-drawer").count()) > 0;
  await page.screenshot({ path: "/tmp/v-map-drawer.png" });
}

// --- guides: pagination ---
await page.goto(`${base}/inspace/guides`, { waitUntil: "networkidle" }).catch(() => {});
await page.waitForTimeout(4000);
const guideItems = await page.locator(".guide-list > li").count();
const resultBar = await page.locator(".directory-result-bar p").first().textContent().catch(() => null);
const pagerButtons = await page.locator(".directory-pagination .pagination-page").count();
await page.screenshot({ path: "/tmp/v-guides.png", fullPage: false });

// go to page 3
let page3 = null;
if (pagerButtons > 2) {
  await page.locator(".directory-pagination .pagination-page").nth(2).click().catch(() => {});
  await page.waitForTimeout(2500);
  page3 = { items: await page.locator(".guide-list > li").count(), bar: await page.locator(".directory-result-bar p").first().textContent().catch(() => null) };
}

// search
await page.locator(".guide-search-row input").fill("京都").catch(() => {});
await page.locator('.guide-search-row button[type="submit"]').click().catch(() => {});
await page.waitForTimeout(2500);
const searchBar = await page.locator(".directory-result-bar p").first().textContent().catch(() => null);
const searchItems = await page.locator(".guide-list > li").count();

// --- explore ---
await page.goto(`${base}/inspace/explore`, { waitUntil: "networkidle" }).catch(() => {});
await page.waitForTimeout(3000);
const exploreItems = await page.locator(".space-directory-list > *").count();
const exploreBar = await page.locator(".directory-result-bar p").first().textContent().catch(() => null);

console.log(JSON.stringify({
  map: { worldPins, worldClusters, afterZoom, drawerOpened },
  guides: { guideItems, resultBar, pagerButtons, page3, search: { searchBar, searchItems } },
  explore: { exploreItems, exploreBar },
  errors: errors.slice(0, 6),
}, null, 2));
await browser.close();
