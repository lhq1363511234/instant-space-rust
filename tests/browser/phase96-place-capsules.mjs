import { chromium } from 'playwright';
import fs from 'fs';

const BASE = 'http://127.0.0.1:3001/inspace';
const SPACE_ID = '10000000-0000-0000-0000-000000000001';
const OUT = 'output/playwright/phase96-place-capsules';
const VIEWPORTS = [
  ['desktop', { width: 1440, height: 900 }],
  ['tablet', { width: 768, height: 1024 }],
  ['phone', { width: 390, height: 844 }],
];

fs.mkdirSync(OUT, { recursive: true });
const browser = await chromium.launch({ args: ['--no-sandbox'] });
const results = [];

for (const [name, viewport] of VIEWPORTS) {
  const context = await browser.newContext({ viewport });
  const page = await context.newPage();
  const errors = [];
  const failedRequests = [];

  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => {
    if (message.type() === 'error') errors.push(message.text());
  });
  page.on('requestfailed', request => {
    if (!request.url().includes('favicon')) {
      failedRequests.push(`${request.method()} ${request.url()} ${request.failure()?.errorText || ''}`);
    }
  });

  const worldUrl = `${BASE}/world/${SPACE_ID}`;
  await page.goto(worldUrl, { waitUntil: 'networkidle', timeout: 60000 });
  await page.locator('[data-world-runtime]').waitFor({ timeout: 30000 });

  const capsuleObject = page.locator('[data-world-fallback-object]', { hasText: /埋信处|Capsule/ }).first();
  // The text fallback is intentionally hidden when the interactive world is
  // ready; trigger its existing listener to exercise the same accessible path.
  await capsuleObject.evaluate(button => button.click());
  const worldPrompt = page.locator('[data-world-action]');
  await worldPrompt.waitFor({ state: 'visible', timeout: 30000 });
  await worldPrompt.click();
  const sheet = page.locator('[data-world-sheet]');
  await sheet.waitFor({ state: 'visible' });

  const action = sheet.locator('.world-primary-action', { hasText: /查看埋信处|capsule grove/i });
  await action.click();
  const dialog = page.locator('.space-experience-dialog');
  await dialog.waitFor({ state: 'visible', timeout: 30000 });
  await dialog.locator('#space-capsules').waitFor({ state: 'visible', timeout: 30000 });

  const capsuleResult = await page.evaluate(worldUrl => {
    const locks = [...document.querySelectorAll('.space-capsule-locks li')];
    const dialog = document.querySelector('.space-experience-dialog');
    return {
      urlUnchanged: location.href === worldUrl,
      lockCount: locks.length,
      lockText: locks.map(lock => lock.textContent.replace(/\s+/g, ' ').trim()),
      presenceVisible: Boolean(dialog?.querySelector('.presence-bar')),
      overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
      dialogOverflow: dialog ? dialog.scrollWidth - dialog.clientWidth : null,
    };
  }, worldUrl);
  await page.screenshot({ path: `${OUT}/${name}-capsules.png`, fullPage: false });

  await dialog.locator('.space-experience-close').click();
  await dialog.waitFor({ state: 'detached' });
  await page.goto(`${BASE}/spaces/${SPACE_ID}`, { waitUntil: 'networkidle', timeout: 60000 });
  await page.locator('.space-entry-row', { hasText: /故事|Stories/ }).first().click();
  const storyPath = page.locator('.space-capsule-path');
  await storyPath.waitFor({ state: 'visible' });
  const storyResult = await page.evaluate(() => ({
    lightEntryCount: document.querySelectorAll('.space-capsule-path').length,
    embeddedShelfCount: document.querySelectorAll('.space-panel-open .capsule-shelf').length,
    overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    entryHeight: document.querySelector('.space-capsule-path')?.getBoundingClientRect().height || 0,
  }));

  await storyPath.click();
  await page.locator('#space-capsules').waitFor({ state: 'visible' });
  const routeSwitchesToCapsules = await page.locator('#space-capsules').isVisible();

  const ok = capsuleResult.urlUnchanged
    && capsuleResult.lockCount === 3
    && capsuleResult.presenceVisible
    && capsuleResult.overflow <= 0
    && capsuleResult.dialogOverflow <= 0
    && storyResult.lightEntryCount === 1
    && storyResult.embeddedShelfCount === 0
    && storyResult.overflow <= 0
    && storyResult.entryHeight >= 44
    && routeSwitchesToCapsules
    && errors.length === 0
    && failedRequests.length === 0;

  results.push({ name, viewport, ok, capsuleResult, storyResult, routeSwitchesToCapsules, errors, failedRequests });
  console.log(ok ? 'PASS' : 'FAIL', name, JSON.stringify(results.at(-1)));
  await context.close();
}

await browser.close();
const report = {
  milestone: 'Phase 9.6 first place capsule loop',
  testedAt: new Date().toISOString(),
  results,
};
fs.writeFileSync(`${OUT}/report.json`, `${JSON.stringify(report, null, 2)}\n`);

if (results.some(result => !result.ok)) process.exitCode = 1;
