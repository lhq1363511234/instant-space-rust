import { chromium } from 'playwright';
const b = await chromium.launch();
const shots = [
  ['home-1440', 'https://opctoai.com/inspace', 1440, 900],
  ['home-ipad-land', 'https://opctoai.com/inspace', 1180, 820],
  ['home-390', 'https://opctoai.com/inspace', 390, 844],
  ['space-ipad-land', 'https://opctoai.com/inspace/spaces/10000000-0000-0000-0000-000000000001', 1180, 820],
  ['space-1440', 'https://opctoai.com/inspace/spaces/10000000-0000-0000-0000-000000000001', 1440, 900],
  ['space-390', 'https://opctoai.com/inspace/spaces/10000000-0000-0000-0000-000000000001', 390, 844],
];
for (const [name, url, w, h] of shots) {
  const ctx = await b.newContext({ viewport: { width: w, height: h }, deviceScaleFactor: 1 });
  await ctx.addCookies([{ name:'instant_session', value:'qa-token-fullstack-1', domain:'opctoai.com', path:'/' }]);
  const p = await ctx.newPage();
  const errs = [];
  p.on('console', m => m.type()==='error' && errs.push(m.text()));
  await p.goto(url, { waitUntil:'networkidle', timeout:60000 });
  await p.waitForTimeout(2500);
  await p.screenshot({ path:`output/playwright/${name}.png`, fullPage: true });
  const sw = await p.evaluate(() => document.documentElement.scrollWidth);
  console.log(name, 'sw=', sw, 'vw=', w, 'errs=', errs.length);
  await ctx.close();
}
await b.close();
