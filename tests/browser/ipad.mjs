// iPad device-emulated screenshot. node tests/browser/ipad.mjs <url> <label> <orientation>
import { chromium, devices } from "playwright";

const url = process.argv[2] || "http://127.0.0.1:3001/inspace";
const label = process.argv[3] || "ipad";
const orientation = process.argv[4] || "landscape"; // landscape | portrait

// iPad Pro 11: 1194x834. Playwright device sets isMobile+hasTouch => pointer:coarse.
const base = devices["iPad Pro 11 landscape"];
const ctxOpts = orientation === "portrait"
  ? { ...devices["iPad Pro 11"] }
  : { ...base };

const browser = await chromium.launch();
const context = await browser.newContext(ctxOpts);
const page = await context.newPage();
await page.goto(url, { waitUntil: "networkidle", timeout: 20000 }).catch(() => {});
await page.waitForTimeout(1500);

// report pointer media + hero grid
const diag = await page.evaluate(() => {
  const coarse = window.matchMedia("(pointer: coarse)").matches;
  const land = window.matchMedia("(orientation: landscape)").matches;
  const hero = document.querySelector(".home-hero-card");
  const cs = hero ? getComputedStyle(hero) : null;
  return {
    pointerCoarse: coarse,
    landscape: land,
    vw: window.innerWidth,
    vh: window.innerHeight,
    heroCols: cs ? cs.gridTemplateColumns : null,
    heroLeft: cs ? cs.left : null,
    heroTop: cs ? cs.top : null,
  };
});
console.log(JSON.stringify(diag));

const out = `/tmp/shot-${label}-${orientation}.png`;
await page.screenshot({ path: out });
console.log("saved", out);
await browser.close();
