// Ad-hoc screenshot tool: node tests/browser/shot.mjs <url> <label> <w> <h>
import { chromium } from "playwright";

const url = process.argv[2] || "http://127.0.0.1:3001/inspace";
const label = process.argv[3] || "shot";
const w = parseInt(process.argv[4] || "1194", 10);
const h = parseInt(process.argv[5] || "834", 10);

const browser = await chromium.launch();
const page = await browser.newPage({
  viewport: { width: w, height: h },
  deviceScaleFactor: 2,
});
await page.goto(url, { waitUntil: "networkidle", timeout: 20000 }).catch(() => {});
await page.waitForTimeout(1500);
const out = `/tmp/shot-${label}-${w}x${h}.png`;
await page.screenshot({ path: out, fullPage: false });
console.log("saved", out);
await browser.close();
