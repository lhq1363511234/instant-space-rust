import { chromium } from 'playwright';
const b=await chromium.launch();
const c=await b.newContext({viewport:{width:1280,height:1000},geolocation:{latitude:31.2397,longitude:121.4998},permissions:['geolocation']});
await c.addCookies([{name:'instant_session',value:'qa-token-fullstack-1',domain:'opctoai.com',path:'/'}]);
const p=await c.newPage();
await p.goto('https://opctoai.com/inspace/spaces/10000000-0000-0000-0000-000000000001',{waitUntil:'networkidle'});
await p.waitForTimeout(2500);
// 封一封不去开的信
await p.locator('.capsule-shelf-head button').click(); await p.waitForTimeout(600);
const ins=p.locator('.capsule-composer input[type=text]');
await ins.nth(0).click(); await ins.nth(0).pressSequentially('玻璃验收',{delay:10});
const cta=p.locator('.capsule-composer textarea');
await cta.click(); await cta.pressSequentially('这封不拆，只看它蒙没蒙一层。',{delay:10});
await ins.nth(1).click(); await ins.nth(1).pressSequentially('留着',{delay:10});
await p.waitForTimeout(300);
await p.locator('.capsule-composer button[type=submit]').click();
await p.waitForTimeout(3500);
const r=await p.evaluate(()=>{
  const s=document.querySelector('.capsule-card.is-sealed');
  const o=document.querySelector('.capsule-card.is-open');
  return {sealedBf: s?getComputedStyle(s).backdropFilter:'NO-SEALED',
          openBf: o?getComputedStyle(o).backdropFilter:'NO-OPEN'};
});
console.log(r);
console.log(r.sealedBf.includes('blur')?'PASS  未开启的信有宣纸玻璃':'FAIL  '+r.sealedBf);
console.log(r.openBf==='none'||r.openBf==='NO-OPEN'?'PASS  已拆的信不蒙玻璃':'FAIL  '+r.openBf);
await p.screenshot({path:'/tmp/qa-sealed-glass.png'});
await b.close();
