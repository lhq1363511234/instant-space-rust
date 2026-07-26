import { chromium } from 'playwright';
const b = await chromium.launch();
const url = 'https://opctoai.com/inspace';
for (const [name, w, h] of [['home-1440',1440,900],['home-ipad-land',1180,820],['home-390',390,844]]) {
  const ctx = await b.newContext({ viewport:{width:w,height:h}, deviceScaleFactor:1 });
  const p = await ctx.newPage();
  const errs=[]; p.on('console', m=>m.type()==='error'&&errs.push(m.text()));
  await p.goto(url,{waitUntil:'networkidle',timeout:60000});
  await p.waitForTimeout(2000);
  const sup = await p.evaluate(()=>CSS.supports('animation-timeline','view()'));
  const sw = await p.evaluate(()=>document.documentElement.scrollWidth);
  const dh = await p.evaluate(()=>document.documentElement.scrollHeight);
  console.log(name,'timelineSupport=',sup,'sw=',sw,'vw=',w,'docH=',dh,'errs=',errs.length, errs.slice(0,3));
  for (const frac of [0, 0.25, 0.5, 0.75, 1]) {
    await p.evaluate(f=>window.scrollTo(0, (document.documentElement.scrollHeight-innerHeight)*f), frac);
    await p.waitForTimeout(700);
    await p.screenshot({ path:`output/playwright/motion-${name}-${Math.round(frac*100)}.png` });
  }
  // measure motion actually applied
  const m = await p.evaluate(()=>{
    const g = s => { const e=document.querySelector(s); if(!e) return null; const c=getComputedStyle(e); return {t:c.transform,o:c.opacity}; };
    const gb = s => { const e=document.querySelector(s); if(!e) return null; const c=getComputedStyle(e,'::before'); return {t:c.transform}; };
    return { rule: gb('.inspace-home-modules'), heroCopy: g('.survey-hero-copy'), sheet: g('.survey-sheet'), stagesRule: gb('.survey-stages'), row1: g('.survey-log tbody tr') };
  });
  console.log('  at bottom:', JSON.stringify(m));
  await ctx.close();
}
await b.close();
