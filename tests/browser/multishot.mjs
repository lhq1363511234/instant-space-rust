import { chromium } from 'playwright-core';
const shots = [
  ['explore', 'http://127.0.0.1:3001/inspace/explore'],
  ['guides', 'http://127.0.0.1:3001/inspace/guides'],
  ['space', 'http://127.0.0.1:3001/inspace/spaces/60358e0f-6ddb-41de-9595-4b25e06e4717'],
];
const browser = await chromium.launch();
const ctx = await browser.newContext({ viewport: { width: 390, height: 834 }, deviceScaleFactor: 2 });
const page = await ctx.newPage();
for (const [name, url] of shots) {
  await page.goto(url, { waitUntil: 'networkidle' });
  await page.waitForTimeout(1200);
  await page.screenshot({ path: `/tmp/shot-output/${name}-mobile.png`, fullPage: true });
  console.log('saved', name);
}
await browser.close();
