// Login then screenshot an authed page.
// node tests/browser/auth-shot.mjs <email> <password> <path> <label> <w> <h>
import { chromium } from "playwright";

const [email, password, path, label, w, h] = [
  process.argv[2], process.argv[3], process.argv[4] || "/inspace/guides/new",
  process.argv[5] || "authed", parseInt(process.argv[6] || "1440", 10), parseInt(process.argv[7] || "900", 10),
];

const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: w, height: h }, deviceScaleFactor: 2 });
const page = await ctx.newPage();

await page.goto("http://127.0.0.1:3001/inspace/login", { waitUntil: "networkidle" }).catch(() => {});
await page.waitForTimeout(1200);
await page.fill('input[type="email"]', email).catch(() => {});
await page.fill('input[type="password"]', password).catch(() => {});
await page.click('button[type="submit"]').catch(() => {});
await page.waitForTimeout(2000);

await page.goto("http://127.0.0.1:3001" + path, { waitUntil: "networkidle" }).catch(() => {});
await page.waitForTimeout(1800);
const out = `/tmp/shot-${label}-${w}x${h}.png`;
await page.screenshot({ path: out });
console.log("saved", out, "| url:", page.url());
await browser.close();
