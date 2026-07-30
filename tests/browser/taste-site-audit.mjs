import { chromium } from 'playwright';
import { mkdirSync, writeFileSync } from 'fs';

const BASE = 'https://opctoai.com/inspace';
const HOSTED = '2d5fbb15-c108-57b0-a42d-857702fac329';
const RECRUITING = '3981c89e-4c6a-570a-9046-46491c067163';
const GUIDE = '45ff3a1c-6aaa-51a3-9d90-4dbf21420508';
const routes = [
  ['home', ''],
  ['about', '/about'],
  ['explore', '/explore'],
  ['guides', '/guides'],
  ['guide-detail', `/guides/${GUIDE}`],
  ['map', '/map'],
  ['space-hosted', `/spaces/${HOSTED}`],
  ['space-recruiting', `/spaces/${RECRUITING}`],
  ['chat', `/spaces/${HOSTED}/chat`],
  ['login', '/login'],
  ['my-spaces-gate', '/my-spaces'],
  ['admin-gate', '/admin'],
  ['admin-home-gate', '/admin/home'],
];
const viewports = [
  ['desktop', { width: 1440, height: 900 }],
  ['ipad', { width: 1024, height: 768 }],
  ['phone', { width: 390, height: 844 }],
];
const OUT = 'output/playwright/taste-audit';
mkdirSync(OUT, { recursive: true });
const browser = await chromium.launch();
const results = [];

for (const [vpName, viewport] of viewports) {
  mkdirSync(`${OUT}/${vpName}`, { recursive: true });
  for (const [name, path] of routes) {
    const context = await browser.newContext({ viewport });
    const page = await context.newPage();
    const consoleErrors = [];
    const requestFailures = [];
    page.on('console', m => { if (m.type() === 'error') consoleErrors.push(m.text()); });
    page.on('requestfailed', r => requestFailures.push(`${r.method()} ${r.url()} ${r.failure()?.errorText || ''}`));
    let loadError = null;
    try {
      await page.goto(`${BASE}${path}`, { waitUntil: 'networkidle', timeout: 70000 });
      await page.waitForTimeout(name === 'map' ? 2200 : 900);
    } catch (e) { loadError = String(e); }

    const metrics = await page.evaluate(() => {
      const visible = el => {
        const r = el.getBoundingClientRect();
        const cs = getComputedStyle(el);
        return r.width > 0 && r.height > 0 && cs.visibility !== 'hidden' && cs.display !== 'none';
      };
      const border = el => {
        const cs = getComputedStyle(el);
        return ['Top','Right','Bottom','Left'].some(x => parseFloat(cs[`border${x}Width`]) > 0 && cs[`border${x}Style`] !== 'none');
      };
      const headings = [...document.querySelectorAll('h1,h2')].filter(visible).map(el => {
        const cs = getComputedStyle(el); const r = el.getBoundingClientRect();
        const lh = parseFloat(cs.lineHeight) || parseFloat(cs.fontSize) * 1.2;
        return { tag: el.tagName, text: (el.textContent || '').trim().slice(0,100), lines: Math.max(1, Math.round(r.height/lh)), font: parseFloat(cs.fontSize) };
      });
      const controls = [...document.querySelectorAll('a,button,input,select,textarea')].filter(visible);
      const smallTargets = controls.map(el => { const r=el.getBoundingClientRect(); return {tag:el.tagName,text:(el.textContent||el.getAttribute('aria-label')||'').trim().slice(0,50),w:Math.round(r.width),h:Math.round(r.height)}; }).filter(x => x.w < 44 || x.h < 44);
      const unlabeled = [...document.querySelectorAll('input,select,textarea')].filter(visible).filter(el => {
        const id=el.id; const aria=el.getAttribute('aria-label')||el.getAttribute('aria-labelledby');
        return !aria && !(id && document.querySelector(`label[for="${CSS.escape(id)}"]`)) && !el.closest('label');
      }).map(el => `${el.tagName.toLowerCase()}#${el.id}.${el.className}`);
      const imagesNoAlt = [...document.querySelectorAll('img')].filter(el => !el.hasAttribute('alt')).length;
      const bordered = [...document.querySelectorAll('body *')].filter(el => visible(el) && border(el));
      const nestedBorders = bordered.filter(el => { let p=el.parentElement; while(p && p!==document.body){ if(border(p)) return true; p=p.parentElement; } return false; }).length;
      const tinyText = [...document.querySelectorAll('p,li,span,small,dt,dd,label')].filter(visible).filter(el => parseFloat(getComputedStyle(el).fontSize) < 12).length;
      const clipped = [...document.querySelectorAll('h1,h2,h3,p,a,button,label')].filter(visible).filter(el => el.scrollWidth > el.clientWidth + 2 || el.scrollHeight > el.clientHeight + 3).length;
      const text=(document.body.innerText||'');
      return {
        url: location.href,
        title: document.title,
        overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
        h1Count: document.querySelectorAll('h1').length,
        headings,
        smallTargets: smallTargets.slice(0,20),
        unlabeled,
        imagesNoAlt,
        borderCount: bordered.length,
        nestedBorders,
        tinyText,
        clipped,
        emDashCount: (text.match(/—/g)||[]).length,
        numberedLabels: [...document.querySelectorAll('p,span,small')].filter(visible).map(x=>(x.textContent||'').trim()).filter(x=>/^(0?\d\s*[·/]|section\s+\d|stage\s+\d)/i.test(x)).slice(0,20),
        animations: document.getAnimations().filter(a => a.playState === 'running').length,
      };
    }).catch(() => ({}));
    const shot = `${OUT}/${vpName}/${name}.png`;
    await page.screenshot({ path: shot, fullPage: false }).catch(()=>{});
    results.push({ viewport: vpName, route: name, path, loadError, consoleErrors, requestFailures, metrics, shot });
    console.log(`${vpName.padEnd(7)} ${name.padEnd(18)} overflow=${metrics.overflow ?? '?'} h1=${metrics.h1Count ?? '?'} small=${metrics.smallTargets?.length ?? '?'} borders=${metrics.borderCount ?? '?'} nested=${metrics.nestedBorders ?? '?'} errs=${consoleErrors.length}/${requestFailures.length}`);
    await context.close();
  }
}

// Reduced-motion proof on the most motion-heavy route.
const rm = await browser.newContext({ viewport: { width: 390, height: 844 }, reducedMotion: 'reduce' });
const rp = await rm.newPage();
await rp.goto(BASE, { waitUntil: 'networkidle', timeout: 70000 });
await rp.waitForTimeout(600);
const reduced = await rp.evaluate(() => ({ running: document.getAnimations().filter(a=>a.playState==='running').length, hyperSticky: getComputedStyle(document.querySelector('.home-hyperframes-sticky')).position }));
await rp.screenshot({ path: `${OUT}/phone/home-reduced-motion.png`, fullPage: false });
await rm.close();

writeFileSync(`${OUT}/report.json`, JSON.stringify({ generatedAt: new Date().toISOString(), results, reduced }, null, 2));
await browser.close();
console.log(`REPORT ${OUT}/report.json reduced=${JSON.stringify(reduced)}`);
