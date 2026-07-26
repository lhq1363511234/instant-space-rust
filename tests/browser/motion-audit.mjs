/* v15 动效验收：不看源码，看浏览器算出来的值。 */
import { chromium } from 'playwright';
const SPACE='10000000-0000-0000-0000-000000000001';
const BASE='https://opctoai.com/inspace';
const b=await chromium.launch();
const c=await b.newContext({viewport:{width:1280,height:1000},
  geolocation:{latitude:31.2397,longitude:121.4998},permissions:['geolocation']});
await c.addCookies([{name:'instant_session',value:'qa-token-fullstack-1',domain:'opctoai.com',path:'/'}]);
const p=await c.newPage();
const errs=[]; p.on('pageerror',e=>errs.push(e.message));
p.on('console',m=>{if(m.type()==='error')errs.push('console: '+m.text())});
const fail=[]; const check=(n,ok,got)=>{console.log((ok?'PASS  ':'FAIL  ')+n+(ok?'':`  got=${got}`)); if(!ok)fail.push(n);};

await p.goto(`${BASE}/spaces/${SPACE}`,{waitUntil:'networkidle'});
await p.waitForTimeout(2500);
await p.locator('#space-traces').scrollIntoViewIfNeeded();
await p.waitForTimeout(600);

// ① 输入框：底线聚焦，不是 outline 方框
const inp=p.locator('.presence-code input').first();
const before=await inp.evaluate(el=>{const s=getComputedStyle(el);return {bw:s.borderBottomWidth, top:s.borderTopWidth};});
check('输入框只有底线（无上边框）', before.top==='0px', JSON.stringify(before));
await inp.click(); await p.waitForTimeout(600);
const lineScale=await p.locator('.presence-code label.field-label').first()
  .evaluate(el=>getComputedStyle(el,'::after').transform);
check('聚焦时朱红底线展开(scaleX=1)', /matrix\(1,/.test(lineScale)||lineScale==='none', lineScale);
const labelColor=await p.locator('.presence-code label.field-label > span').first()
  .evaluate(el=>getComputedStyle(el).color);
check('聚焦时标签转朱红', labelColor.includes('178, 58, 41'), labelColor);

// ② 按钮：四段线 pseudo 存在
const btn=p.locator('.presence-code button[type=submit]').first();
const w0=await btn.evaluate(el=>getComputedStyle(el,'::before').width);
await btn.hover(); await p.waitForTimeout(600);
const w1=await btn.evaluate(el=>getComputedStyle(el,'::before').width);
check('hover 时线延展', parseFloat(w1)>parseFloat(w0)+8, `${w0}→${w1}`);

// ③ 胶囊卡片：宣纸玻璃 + 左缘标记线
// 宣纸玻璃只加在未开启的信上——已拆的信不该再蒙一层。
const sealed=p.locator('.capsule-card.is-sealed').first();
if (await sealed.count()) {
  const g=await sealed.evaluate(el=>getComputedStyle(el).backdropFilter);
  check('未开启的信有宣纸玻璃', g.includes('blur'), g);
  const go=await p.locator('.capsule-card.is-open').first()
    .evaluate(el=>getComputedStyle(el).backdropFilter).catch(()=>'none');
  check('已拆的信不蒙玻璃', go==='none', go);
} else { console.log('SKIP  现场无未开启的信'); }

const cap=p.locator('.capsule-card').first();
if (await cap.count()) {
  const m0=await cap.evaluate(el=>getComputedStyle(el,'::before').transform);
  await cap.hover(); await p.waitForTimeout(600);
  const m1=await cap.evaluate(el=>getComputedStyle(el,'::before').transform);
  check('hover 时左缘朱红线长出', m0!==m1, `${m0} → ${m1}`);
} else { console.log('SKIP  无胶囊卡片'); }

await p.screenshot({path:'/tmp/qa-motion-desktop.png'});
await p.setViewportSize({width:390,height:844});
await p.waitForTimeout(900);
await p.locator('#space-traces').scrollIntoViewIfNeeded();
await p.waitForTimeout(400);
const of=await p.evaluate(()=>document.documentElement.scrollWidth-document.documentElement.clientWidth);
check('手机零横向溢出', of<=0, of);
await p.screenshot({path:'/tmp/qa-motion-mobile.png'});
console.log('errors:',errs.length,errs.slice(0,3));
console.log(fail.length?'\nFAILED: '+fail.join(' | '):'\nALL PASS');
await b.close();
