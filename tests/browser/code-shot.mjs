import { chromium } from 'playwright';
const b=await chromium.launch();
for (const [name,w,h] of [['desktop',1440,900],['mobile',390,844]]) {
  const c=await b.newContext({viewport:{width:w,height:h}});
  await c.addCookies([{name:'instant_session',value:'qa-token-fullstack-1',domain:'opctoai.com',path:'/'}]);
  const p=await c.newPage();
  await p.goto('https://opctoai.com/inspace/spaces/10000000-0000-0000-0000-000000000001',{waitUntil:'networkidle'});
  await p.waitForTimeout(2500);
  await p.locator('.presence-bar').scrollIntoViewIfNeeded();
  await p.waitForTimeout(600);
  await p.screenshot({path:`/tmp/qa-code-${name}.png`});
  console.log(name,'overflow:',await p.evaluate(()=>document.documentElement.scrollWidth-document.documentElement.clientWidth));
  await c.close();
}
await b.close();
