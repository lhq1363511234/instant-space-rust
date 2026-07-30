import { chromium } from 'playwright';
const BASE = 'https://opctoai.com/inspace';
const OUT = 'output/playwright';
import { mkdirSync } from 'fs';
mkdirSync(OUT, { recursive: true });
const b = await chromium.launch();
const views = [['desktop',{width:1440,height:900}],['phone',{width:390,height:844}]];
for (const [name, viewport] of views) {
  const c = await b.newContext({ viewport });
  const p = await c.newPage();
  const errs = [];
  p.on('console', m => { if (m.type()==='error') errs.push(m.text()); });
  await p.goto(BASE, { waitUntil:'networkidle' });
  await p.waitForTimeout(1200);
  // hero shot
  await p.screenshot({ path: `${OUT}/home-${name}-hero.png` });
  const overflow = await p.evaluate(()=>document.documentElement.scrollWidth-document.documentElement.clientWidth);
  // scroll to cases
  const cases = await p.$('.survey-cases');
  if (cases) { await cases.scrollIntoViewIfNeeded(); await p.waitForTimeout(600); await p.screenshot({ path:`${OUT}/home-${name}-cases.png` }); }
  const heroTitle = await p.$eval('#inspace-home-title', el=>el.textContent).catch(()=>'(none)');
  const caseCount = await p.$$eval('.survey-case', els=>els.length);
  console.log(`${name} overflow=${overflow} cases=${caseCount} errs=${errs.length} title="${heroTitle}"`);
  await c.close();
}
await b.close();
