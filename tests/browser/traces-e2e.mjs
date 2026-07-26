import { chromium } from 'playwright';

const SPACE = '10000000-0000-0000-0000-000000000001';
const BASE = 'https://opctoai.com/inspace';
const LAT = 31.2397, LNG = 121.4998;

const browser = await chromium.launch();
const ctx = await browser.newContext({
  viewport: { width: 1280, height: 900 },
  geolocation: { latitude: LAT, longitude: LNG },
  permissions: ['geolocation'],
});
await ctx.addCookies([{ name: 'instant_session', value: 'qa-token-fullstack-1', domain: 'opctoai.com', path: '/' }]);
const page = await ctx.newPage();
const errors = [];
page.on('console', m => { if (m.type() === 'error') errors.push(m.text()); });
page.on('pageerror', e => errors.push('pageerror: ' + e.message));

await page.goto(`${BASE}/spaces/${SPACE}?via=qr`, { waitUntil: 'networkidle' });
await page.waitForTimeout(2500);

const traces = page.locator('#space-traces');
console.log('traces section visible:', await traces.isVisible());
console.log('presence badge:', (await page.locator('.presence-badge').first().innerText().catch(()=>'NONE')).trim());

// Leave a trace
const stamp = 'QA留痕 ' + Date.now();
const ta = page.locator('.trace-composer textarea');
await ta.click();
await ta.pressSequentially(stamp.slice(0, 12), { delay: 20 });
await page.waitForTimeout(500);
console.log('submit enabled after typing:', await page.locator('.trace-composer button[type=submit]').isEnabled());
await page.locator('.trace-weather-field input').click();
await page.locator('.trace-weather-field input').pressSequentially('晴', { delay: 20 });
await page.locator('.trace-composer button[type=submit]').click();
await page.waitForTimeout(3000);
const wrote = await page.locator('.trace-list').innerText().catch(()=>'');
console.log('trace persisted:', wrote.includes(stamp.slice(0,12)));
console.log('proof label:', (await page.locator('.trace-entry').first().locator('.trace-proof').innerText().catch(()=>'NONE')).trim());

// Seal a capsule
await page.locator('.capsule-shelf-head button').click();
await page.waitForTimeout(500);
const rec = 'QA收信人 ' + Date.now();
const inputs = page.locator('.capsule-composer input[type=text]');
await inputs.nth(0).fill(rec);
await page.locator('.capsule-composer textarea').fill('这是一封测试信，写给之后来的人。');
await inputs.nth(1).fill('黄山日出');
await page.locator('.capsule-composer button[type=submit]').click();
await page.waitForTimeout(3000);
const shelf = await page.locator('.capsule-list').innerText().catch(()=>'');
console.log('capsule sealed:', shelf.includes(rec));

// Open with wrong passphrase
const card = page.locator('.capsule-card.is-sealed').first();
await card.locator('button:has-text("这是给我的")').click();
await page.waitForTimeout(400);
await card.locator('.capsule-attempt input[type=text]').fill('错的口令');
await card.locator('.capsule-attempt button.button-primary').click();
await page.waitForTimeout(2500);
console.log('wrong-pass result:', (await card.locator('.capsule-result').innerText().catch(()=>'NONE')).trim());

// Open with right passphrase
await card.locator('.capsule-attempt input[type=text]').fill('黄山日出');
await card.locator('.capsule-attempt button.button-primary').click();
await page.waitForTimeout(2500);
console.log('opened letter:', (await card.locator('.capsule-letter-body').innerText().catch(()=>'NONE')).trim());

await page.screenshot({ path: '/tmp/qa-traces-desktop.png', fullPage: false });
await page.setViewportSize({ width: 390, height: 844 });
await page.waitForTimeout(800);
await traces.scrollIntoViewIfNeeded();
await page.screenshot({ path: '/tmp/qa-traces-mobile.png' });
const ow = await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
console.log('mobile horizontal overflow px:', ow);

console.log('console errors:', errors.length, errors.slice(0,5));
await browser.close();
