import { chromium } from "playwright";
const base = process.env.BASE_URL || "https://opctoai.com";
const targets = [
  ["home-1440", "/inspace", 1440, 900],
  ["home-ipad-land", "/inspace", 1024, 768],
  ["home-ipad-port", "/inspace", 768, 1024],
  ["home-390", "/inspace", 390, 844],
];
const browser = await chromium.launch();
const out = [];
for (const [name, path, w, h] of targets) {
  const ctx = await browser.newContext({ viewport: { width: w, height: h }, deviceScaleFactor: 1 });
  const page = await ctx.newPage();
  const errors = [];
  page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()); });
  await page.goto(`${base}${path}`, { waitUntil: "networkidle" }).catch(() => {});
  await page.waitForTimeout(2500);
  const overflow = await page.evaluate(() => ({ sw: document.documentElement.scrollWidth, vw: window.innerWidth }));
  await page.screenshot({ path: `output/playwright/${name}.png` });
  out.push({ name, path, viewport: `${w}x${h}`, overflow, errors: errors.slice(0,3) });
  await ctx.close();
}
console.log(JSON.stringify(out, null, 2));
await browser.close();
