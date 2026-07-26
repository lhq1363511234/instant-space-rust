import { chromium } from 'playwright';
const shots = [
  { name: 'home-mobile', url: 'http://127.0.0.1:3001/inspace', vp: { width: 390, height: 844 } },
  { name: 'home-desktop', url: 'http://127.0.0.1:3001/inspace', vp: { width: 1440, height: 900 } },
];
const browser = await chromium.launch();
for (const s of shots) {
  const page = await browser.newPage({ viewport: s.vp });
  await page.goto(s.url, { waitUntil: 'networkidle' });
  await page.waitForTimeout(1200);
  await page.screenshot({ path: `output/audit-${s.name}.png`, fullPage: true });
  console.log('shot', s.name);
}
await browser.close();
