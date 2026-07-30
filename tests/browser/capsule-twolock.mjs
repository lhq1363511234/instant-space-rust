/* v15: 胶囊双口令。
   证明两件事：
   ① 人就站在空间里（GPS 在半径内），但没给现场口令 → 必须 PresenceRequired。
      改动之前，坐标够近就直接放行，这条会失败。
   ② 给了现场口令但胶囊主口令错 → WrongPassphrase（第二把锁独立生效）。
   ③ 两把都对 → 开信。 */
import { chromium } from 'playwright';

const SPACE='10000000-0000-0000-0000-000000000001';
const BASE='https://opctoai.com/inspace';
const CODE='481902';
const PASS='双锁'+Date.now().toString().slice(-5);

const b=await chromium.launch();
// 就站在外滩，GPS 完全在半径内。
const c=await b.newContext({viewport:{width:1280,height:1000},
  geolocation:{latitude:31.2397,longitude:121.4998},permissions:['geolocation']});
await c.addCookies([{name:'instant_session',value:'qa-token-fullstack-1',domain:'opctoai.com',path:'/'}]);
const p=await c.newPage();
const errs=[]; p.on('pageerror',e=>errs.push(e.message));
p.on('console',m=>{if(m.type()==='error')errs.push('console: '+m.text())});
const fail=[];
const check=(name,ok,got)=>{ console.log((ok?'PASS  ':'FAIL  ')+name+(ok?'':`  got=${got}`)); if(!ok)fail.push(name); };

await p.goto(`${BASE}/spaces/${SPACE}`,{waitUntil:'networkidle'});
await p.waitForTimeout(2500);

// 先用定位建立"位置已确认"，注意这在改动后不足以开信。
await p.locator('button:has-text("验证当前位置")').first().click().catch(()=>{});
await p.waitForTimeout(2000);

// 埋胶囊本身也需要现场口令 + 定位。
const createCode=p.locator('.presence-code input');
await createCode.fill(CODE);
await p.locator('.presence-code button[type=submit]').click();
await p.waitForTimeout(1800);

// 造一个胶囊
await p.locator('.capsule-shelf-head button').click();
await p.waitForTimeout(600);
const ins=p.locator('.capsule-composer input[type=text]');
await ins.nth(0).click(); await ins.nth(0).pressSequentially('双锁测试',{delay:10});
const cta=p.locator('.capsule-composer textarea');
await cta.click(); await cta.pressSequentially('两把锁都开了才读得到。',{delay:10});
await ins.nth(1).click(); await ins.nth(1).pressSequentially(PASS,{delay:10});
await p.waitForTimeout(300);
await p.locator('.capsule-composer button[type=submit]').click();
await p.waitForTimeout(3500);

// 重载后清空现场口令状态，专门验证“只有定位不能开信”。
await p.reload({waitUntil:'networkidle'});
await p.waitForTimeout(1800);
await p.locator('button:has-text("验证当前位置")').first().click().catch(()=>{});
await p.waitForTimeout(1800);
const card=p.locator('.capsule-card.is-sealed').first();
await card.locator('button:has-text("这是给我的")').click();
await p.waitForTimeout(600);

// 文案必须讲"两把锁"，不能再说定位能开
const line=(await card.locator('.capsule-presence').innerText()).replace(/\s+/g,' ');
check('提示讲清两把锁', line.includes('两把锁'), line.slice(0,60));
check('不再暗示定位可开信', !line.includes('或者用定位'), line.slice(0,60));

// ① 站在原地 + 正确胶囊口令，但没填现场口令 → 必须拒
const pin=card.locator('.capsule-attempt input[type=text]');
await pin.click(); await pin.pressSequentially(PASS,{delay:20});
await p.waitForTimeout(300);
await card.locator('.capsule-attempt button.button-primary').click();
await p.waitForTimeout(3000);
const r1=(await card.locator('.capsule-result, .capsule-letter-body').first().innerText().catch(()=>'NONE')).trim();
check('在场但无现场口令 → 拒绝', r1.includes('现场口令'), r1.slice(0,60));
check('在场但无现场口令 → 没开信', (await card.locator('.capsule-letter-body').count())===0, 'letter shown');

// ② 填对现场口令，但胶囊口令错 → 第二把锁挡住
const field=p.locator('.presence-code input');
await field.click(); await field.pressSequentially(CODE,{delay:20});
await p.waitForTimeout(200);
await p.locator('.presence-code button[type=submit]').click();
await p.waitForTimeout(2500);
check('现场口令已确认', (await p.locator('.presence-badge').first().innerText()).includes('现场口令'), '');

await pin.fill('');
await pin.click(); await pin.pressSequentially('不是这句话',{delay:20});
await p.waitForTimeout(300);
await card.locator('.capsule-attempt button.button-primary').click();
await p.waitForTimeout(3000);
const r2=(await card.locator('.capsule-result').first().innerText().catch(()=>'NONE')).trim();
check('现场口令对但主口令错 → 拒绝', r2.includes('不是那句话'), r2.slice(0,60));

// ③ 两把都对 → 开信
await pin.fill('');
await pin.click(); await pin.pressSequentially(PASS,{delay:20});
await p.waitForTimeout(300);
await card.locator('.capsule-attempt button.button-primary').click();
await p.waitForTimeout(3500);
const body=(await card.locator('.capsule-letter-body').innerText().catch(()=>'NONE')).trim();
check('两把锁齐 → 开信', body.includes('两把锁都开了'), body.slice(0,60));

await p.screenshot({path:'/tmp/qa-twolock.png',fullPage:false});
console.log('errors:',errs.length,errs.slice(0,3));
console.log(fail.length? '\nFAILED: '+fail.join(' | ') : '\nALL PASS');
await b.close();
