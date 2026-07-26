import { chromium } from 'playwright';
const fail=[]; const check=(n,ok,got)=>{console.log((ok?'PASS  ':'FAIL  ')+n+(ok?'':`  got=${got}`)); if(!ok)fail.push(n);};
const b=await chromium.launch();
const p=await (await b.newContext({viewport:{width:1280,height:900}})).newPage();
await p.goto('https://opctoai.com/inspace',{waitUntil:'networkidle'});
await p.waitForTimeout(2000);

const figs=await p.evaluate(()=>[...document.querySelectorAll('.survey-field-plate figure')].map(e=>{
  const s=getComputedStyle(e); return {an:s.animationName, rg:s.animationRange||s.animationRangeStart};}));
check('六张图有 plate-lay 入场', figs.length===6 && figs.every(f=>f.an==='plate-lay'), JSON.stringify(figs.slice(0,2)));
check('六张图错开时机', new Set(figs.map(f=>f.rg)).size===6, figs.map(f=>f.rg).join('|'));

const cap=await p.locator('.survey-field-plate figcaption').first().evaluate(e=>getComputedStyle(e).animationName);
check('说明文字独立入场', cap==='plate-caption', cap);

// 滚到图片带看是否真的从 0 变 1
await p.locator('.survey-field-strip').scrollIntoViewIfNeeded();
await p.waitForTimeout(1200);
const vis=await p.evaluate(()=>[...document.querySelectorAll('.survey-field-plate figure')].map(e=>getComputedStyle(e).opacity));
check('滚到后图片可见', vis.every(v=>parseFloat(v)>0.5), vis.join(','));

// 标题落笔
const h2=await p.locator('.survey-field-head h2').evaluate(e=>getComputedStyle(e).animationName);
check('标题用 ink-set 落笔', h2==='ink-set', h2);
const mk=await p.locator('.survey-kicker-mark').first().evaluate(e=>getComputedStyle(e).animationName);
check('kicker 标记先点上', mk==='mark-drop', mk);

// hover
const plate=p.locator('.survey-field-plate').first();
// figure 的 transform 归入场动画所有，位移由 img 承担
const t0=await plate.locator('img').evaluate(e=>getComputedStyle(e).transform);
await plate.hover(); await p.waitForTimeout(700);
const t1=await plate.locator('img').evaluate(e=>getComputedStyle(e).transform);
check('hover 照片抬起并推近', t0!==t1, `${t0} → ${t1}`);
const fc=await plate.locator('figcaption').evaluate(e=>getComputedStyle(e).borderTopColor);
check('hover 说明线转朱红', fc.includes('178, 58, 41'), fc);

await p.locator('.survey-field-strip').scrollIntoViewIfNeeded();
await p.waitForTimeout(900);
await p.screenshot({path:'/tmp/qa-plates.png'});
await b.close();
console.log(fail.length?'\nFAILED: '+fail.join(' | '):'\nALL PASS');
