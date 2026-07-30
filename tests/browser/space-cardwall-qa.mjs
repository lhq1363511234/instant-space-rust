import { chromium } from 'playwright';
import fs from 'fs';

const BASE = 'https://opctoai.com/inspace';
// Two public spaces: one claimed (故宫, has host), one recruiting (滕王阁, no host).
const SPACES = [
  ['hosted', '2d5fbb15-c108-57b0-a42d-857702fac329'],
  ['recruiting', '3981c89e-4c6a-570a-9046-46491c067163'],
];
const VIEWPORTS = [
  ['desktop', { width: 1440, height: 900 }],
  ['ipad', { width: 1024, height: 768 }],
  ['phone', { width: 390, height: 844 }],
];
const OUT = 'output/playwright/cardwall';
fs.mkdirSync(OUT, { recursive: true });

const b = await chromium.launch();
const failures = [];

for (const [vpName, viewport] of VIEWPORTS) {
  const c = await b.newContext({ viewport });
  for (const [label, id] of SPACES) {
    const p = await c.newPage();
    const errs = [];
    const failed = [];
    p.on('pageerror', e => errs.push(e.message));
    p.on('console', m => { if (m.type() === 'error') errs.push(m.text()); });
    p.on('requestfailed', r => failed.push(`${r.method()} ${r.url()} ${r.failure()?.errorText}`));
    await p.goto(`${BASE}/spaces/${id}`, { waitUntil: 'networkidle', timeout: 60000 });
    await p.waitForTimeout(1200);

    // The editorial place index should be the default view.
    const wall = await p.evaluate(() => {
      const cards = [...document.querySelectorAll('.space-entry-row')];
      const overflow = document.documentElement.scrollWidth - document.documentElement.clientWidth;
      const h1 = document.querySelector('main h1');
      const hr = h1?.getBoundingClientRect();
      return {
        cardCount: cards.length,
        labels: cards.map(c => (c.querySelector('.space-entry-card-label')?.textContent || '').trim()),
        overflow,
        h1Clip: !!hr && (hr.left < 0 || hr.right > innerWidth + 1),
      };
    });

    await p.screenshot({ path: `${OUT}/${label}-${vpName}-wall.png`, fullPage: true });

    // Open the intro card and verify a panel appears with a back control.
    let introOk = false, hostOk = false;
    const introCard = p.locator('.space-entry-row', { hasText: /简介|About/ }).first();
    if (await introCard.count()) {
      await introCard.click();
      await p.waitForTimeout(500);
      introOk = await p.locator('.space-panel-open .space-panel-back').count() > 0
        && await p.locator('.space-fact-wall').count() > 0;
      await p.screenshot({ path: `${OUT}/${label}-${vpName}-intro.png`, fullPage: true });
      // back
      await p.locator('.space-panel-back').first().click();
      await p.waitForTimeout(300);
    }
    // Open host card.
    const hostCard = p.locator('.space-entry-row', { hasText: /主理人|Host/ }).first();
    if (await hostCard.count()) {
      await hostCard.click();
      await p.waitForTimeout(500);
      hostOk = await p.locator('.space-host-panel').count() > 0;
      await p.screenshot({ path: `${OUT}/${label}-${vpName}-host.png`, fullPage: true });
    }

    const realFail = failed.filter(x => !x.includes('favicon'));
    const ok = wall.cardCount >= 5 && wall.overflow <= 0 && !wall.h1Clip
      && introOk && hostOk && errs.length === 0 && realFail.length === 0;
    console.log(ok ? 'PASS' : 'FAIL', vpName, label, JSON.stringify({ ...wall, introOk, hostOk, errs: errs.length, reqfail: realFail.length }));
    if (!ok) failures.push({ vpName, label, wall, introOk, hostOk, errs, failed: realFail });
    await p.close();
  }
  await c.close();
}
await b.close();
console.log(failures.length ? `FAILED ${failures.length}` : 'ALL PASS');
if (failures.length) console.log(JSON.stringify(failures, null, 2));
