import { chromium } from 'playwright';

const SPACE='10000000-0000-0000-0000-000000000001';
const BASE='https://opctoai.com/inspace';
const CODE='481902';

const b=await chromium.launch();
// Beijing: ~1000km away. Nothing but the code can establish presence here.
const c=await b.newContext({viewport:{width:1280,height:900},
  geolocation:{latitude:39.9042,longitude:116.4074},permissions:['geolocation']});
await c.addCookies([{name:'instant_session',value:'qa-token-fullstack-1',domain:'opctoai.com',path:'/'}]);
const p=await c.newPage();
const errs=[]; p.on('pageerror',e=>errs.push(e.message));
p.on('console',m=>{if(m.type()==='error')errs.push('console: '+m.text())});

await p.goto(`${BASE}/spaces/${SPACE}`,{waitUntil:'networkidle'});
await p.waitForTimeout(2500);

const field=p.locator('.presence-code input');
console.log('code field present:', await field.count());

// wrong code must be rejected
await field.click(); await field.pressSequentially('000000',{delay:20});
await p.waitForTimeout(200);
await p.locator('.presence-code button').click();
await p.waitForTimeout(2500);
console.log('badge after wrong code:', (await p.locator('.presence-badge').first().innerText()).trim());
console.log('hint after wrong code:', (await p.locator('.presence-code-hint').innerText()).trim().slice(0,24));

// right code
await field.fill('');
await field.click(); await field.pressSequentially(CODE,{delay:20});
await p.waitForTimeout(200);
await p.locator('.presence-code button').click();
await p.waitForTimeout(2500);
console.log('badge after right code:', (await p.locator('.presence-badge').first().innerText()).trim());
console.log('code field hidden after accept:', (await p.locator('.presence-code input').count())===0);

// a trace written 1000km away, but holding the code, is on-site
const stamp='口令留痕 '+Date.now();
const ta=p.locator('.trace-composer textarea');
await ta.click(); await ta.pressSequentially(stamp.slice(0,12),{delay:20});
await p.waitForTimeout(400);
await p.locator('.trace-composer button[type=submit]').click();
await p.waitForTimeout(3000);
console.log('proof on newest trace:', (await p.locator('.trace-entry').first().locator('.trace-proof').innerText()).trim());

// capsule: seal, then open from far away using only the code
await p.locator('.capsule-shelf-head button').click();
await p.waitForTimeout(500);
const rec='口令测试 '+Date.now();
const ins=p.locator('.capsule-composer input[type=text]');
await ins.nth(0).click(); await ins.nth(0).pressSequentially(rec,{delay:10});
const cta=p.locator('.capsule-composer textarea');
await cta.click(); await cta.pressSequentially('凭现场口令打开的信。',{delay:10});
await ins.nth(1).click(); await ins.nth(1).pressSequentially('同心锁',{delay:10});
await p.waitForTimeout(300);
await p.locator('.capsule-composer button[type=submit]').click();
await p.waitForTimeout(3000);

const card=p.locator('.capsule-card.is-sealed').first();
await card.locator('button:has-text("这是给我的")').click();
await p.waitForTimeout(500);
console.log('capsule presence line:', (await card.locator('.capsule-presence').innerText()).trim().replace(/\s+/g,' ').slice(0,40));
const pin=card.locator('.capsule-attempt input[type=text]');
await pin.click(); await pin.pressSequentially('同心锁',{delay:20});
await p.waitForTimeout(300);
await card.locator('.capsule-attempt button.button-primary').click();
await p.waitForTimeout(3000);
console.log('opened with code from 1000km:', (await card.locator('.capsule-letter-body').innerText().catch(()=>'NONE')).trim());

await p.screenshot({path:'/tmp/qa-onsite-desktop.png'});
await p.setViewportSize({width:390,height:844});
await p.waitForTimeout(800);
await p.locator('#space-traces').scrollIntoViewIfNeeded();
await p.screenshot({path:'/tmp/qa-onsite-mobile.png'});
console.log('mobile overflow:', await p.evaluate(()=>document.documentElement.scrollWidth-document.documentElement.clientWidth));
console.log('errors:',errs.length,errs.slice(0,4));
await b.close();
