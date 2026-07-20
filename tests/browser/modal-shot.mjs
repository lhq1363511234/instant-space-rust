// Login, open create-space modal, screenshot. node modal-shot.mjs <email> <pass> <label> <w> <h>
import { chromium } from "playwright";
const [email, password, label, w, h] = [
  process.argv[2], process.argv[3], process.argv[4] || "modal",
  parseInt(process.argv[5] || "1440", 10), parseInt(process.argv[6] || "900", 10),
];
const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: w, height: h }, deviceScaleFactor: 2 });
const page = await ctx.newPage();
await page.goto("http://127.0.0.1:3001/inspace/login", { waitUntil: "networkidle" }).catch(() => {});
await page.waitForTimeout(1000);
await page.fill('input[type="email"]', email).catch(() => {});
await page.fill('input[type="password"]', password).catch(() => {});
await page.click('button[type="submit"]').catch(() => {});
await page.waitForTimeout(1800);
await page.goto("http://127.0.0.1:3001/inspace/my-spaces", { waitUntil: "networkidle" }).catch(() => {});
await page.waitForTimeout(1500);
// click Create Space / open modal
await page.click('.manage-space-open-btn, button:has-text("Create Space"), button:has-text("创建")').catch((e) => console.log("click:", e.message));
await page.waitForTimeout(1200);
const out = `/tmp/shot-${label}-${w}x${h}.png`;
await page.screenshot({ path: out });
const hasModal = await page.evaluate(() => !!document.querySelector(".manage-space-modal, .modal-card"));
console.log("saved", out, "| modal open:", hasModal);
await browser.close();
