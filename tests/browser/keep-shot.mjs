import { chromium } from 'playwright';
const b=await chromium.launch();
for (const [name,w,h] of [['desktop',1440,900],['mobile',390,844]]) {
  const c=await b.newContext({viewport:{width:w,height:h}});
  const p=await c.newPage();
  const errs=[]; p.on('pageerror',e=>errs.push(e.message));
  await p.goto('https://opctoai.com/inspace',{waitUntil:'networkidle'});
  await p.waitForTimeout(1500);
  const s=p.locator('.survey-keep');
  await s.scrollIntoViewIfNeeded();
  await p.waitForTimeout(1200);
  await p.screenshot({path:`/tmp/qa-keep-${name}.png`});
  const ow=await p.evaluate(()=>document.documentElement.scrollWidth-document.documentElement.clientWidth);
  console.log(name,'overflow:',ow,'errors:',errs.length);
  await c.close();
}
await b.close();
