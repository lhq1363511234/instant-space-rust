import { chromium } from 'playwright';
const BASE = 'https://opctoai.com/inspace';
// recruiting space (no host): 滕王阁
const ID = '3981c89e-4c6a-570a-9046-46491c067163';
const b = await chromium.launch();
const c = await b.newContext({ viewport: { width: 1280, height: 900 } });
const p = await c.newPage();
await p.goto(`${BASE}/spaces/${ID}`, { waitUntil: 'networkidle' });
// click the 主理人 (Host) card
await p.getByText('主理人', { exact: true }).first().click();
await p.waitForTimeout(1500);
await p.screenshot({ path: 'output/playwright/host-claim.png' });
// also phone
const c2 = await b.newContext({ viewport: { width: 390, height: 844 } });
const p2 = await c2.newPage();
await p2.goto(`${BASE}/spaces/${ID}`, { waitUntil: 'networkidle' });
await p2.getByText('主理人', { exact: true }).first().click();
await p2.waitForTimeout(1500);
await p2.screenshot({ path: 'output/playwright/host-claim-phone.png' });
await b.close();
console.log('shots saved');
