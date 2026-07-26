import { chromium } from 'playwright';
const SPACE='10000000-0000-0000-0000-000000000001';
const b=await chromium.launch();
// Beijing, ~1000km from the Bund: right words, wrong place.
const c=await b.newContext({viewport:{width:1280,height:900},geolocation:{latitude:39.9042,longitude:116.4074},permissions:['geolocation']});
await c.addCookies([{name:'instant_session',value:'qa-token-fullstack-1',domain:'opctoai.com',path:'/'}]);
const p=await c.newPage();
const errs=[]; p.on('pageerror',e=>errs.push(e.message));
await p.goto(`https://opctoai.com/inspace/spaces/${SPACE}`,{waitUntil:'networkidle'});
await p.waitForTimeout(2500);

// seal a fresh capsule (no ?via=qr, so presence must come from geolocation)
await p.locator('.capsule-shelf-head button').click();
await p.waitForTimeout(400);
const rec='远距测试 '+Date.now();
const ins=p.locator('.capsule-composer input[type=text]');
await ins.nth(0).click(); await ins.nth(0).pressSequentially(rec,{delay:10});
const ta=p.locator('.capsule-composer textarea');
await ta.click(); await ta.pressSequentially('不该被远程读到的内容',{delay:10});
await ins.nth(1).click(); await ins.nth(1).pressSequentially('外滩夜风',{delay:10});
await p.waitForTimeout(300);
await p.locator('.capsule-composer button[type=submit]').click();
await p.waitForTimeout(3000);

const card=p.locator('.capsule-card.is-sealed').first();
await card.locator('button:has-text("这是给我的")').click();
await p.waitForTimeout(400);
console.log('presence prompt:', (await card.locator('.capsule-presence').innerText()).trim().replace(/\s+/g,' '));
await card.locator('button:has-text("或者用定位")').click();
await p.waitForTimeout(2500);
const inp=card.locator('.capsule-attempt input[type=text]');
await inp.click(); await inp.pressSequentially('外滩夜风',{delay:10});
await p.waitForTimeout(300);
await card.locator('.capsule-attempt button.button-primary').click();
await p.waitForTimeout(3000);
const out=(await card.locator('.capsule-result').innerText().catch(()=>'NONE')).trim();
console.log('right words + wrong place =>', out);
console.log('letter leaked?', await card.locator('.capsule-letter-body').count());
console.log('pageerrors:', errs);
await b.close();
