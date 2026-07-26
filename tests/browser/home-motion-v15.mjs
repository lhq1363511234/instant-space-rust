import { chromium } from 'playwright';
const fail=[]; const check=(n,ok,got)=>{console.log((ok?'PASS  ':'FAIL  ')+n+(ok?'':`  got=${got}`)); if(!ok)fail.push(n);};
const b=await chromium.launch();

// ① Chrome 分支（有 scroll timeline）不能被破坏
{
  const p=await (await b.newContext({viewport:{width:1280,height:900}})).newPage();
  const errs=[]; p.on('pageerror',e=>errs.push(e.message));
  await p.goto('https://opctoai.com/inspace',{waitUntil:'networkidle'});
  await p.waitForTimeout(2000);
  const r=await p.evaluate(()=>({
    h1:getComputedStyle(document.querySelector('.survey-hero h1')).animationName,
    stagesTimeline:getComputedStyle(document.querySelector('.survey-stages > div')).animationTimeline,
  }));
  check('Chrome 仍走滚动联动', r.h1==='home-headline-set' && r.stagesTimeline!=='auto', JSON.stringify(r));
  check('Chrome 无 JS 错误', errs.length===0, errs.slice(0,2));
  await p.context().close();
}

// ② 交互反馈：全浏览器通用，过去首页完全没有
{
  const p=await (await b.newContext({viewport:{width:1280,height:900}})).newPage();
  await p.goto('https://opctoai.com/inspace',{waitUntil:'networkidle'});
  await p.waitForTimeout(2500);

  const sheet=p.locator('.survey-sheet').first();
  if (await sheet.count()) {
    // 首屏这张卡的 transform 归滚动联动所有，指针反馈走边框色。
    const t0=await sheet.evaluate(e=>getComputedStyle(e).borderTopColor);
    await sheet.hover(); await p.waitForTimeout(400);
    const t1=await sheet.evaluate(e=>getComputedStyle(e).borderTopColor);
    check('记录卡 hover 边框转朱红', t0!==t1 && t1.includes('178, 58, 41'), `${t0} → ${t1}`);
    const st=p.locator('.survey-sheet-stamp').first();
    if(await st.count()){
      const s1=await st.evaluate(e=>getComputedStyle(e).transform);
      check('钤印 hover 转正', s1!=='none', s1);
    }
  } else check('记录卡存在', false, 'no .survey-sheet');

  const stage=p.locator('.survey-stages > div').first();
  if (await stage.count()) {
    const m0=await stage.evaluate(e=>getComputedStyle(e,'::before').transform);
    await stage.hover(); await p.waitForTimeout(400);
    const m1=await stage.evaluate(e=>getComputedStyle(e,'::before').transform);
    check('阶段项左缘标记线长出', m0!==m1, `${m0} → ${m1}`);
  }

  const row=p.locator('.survey-log tbody tr').first();
  if (await row.count()) {
    const c0=await row.evaluate(e=>getComputedStyle(e).backgroundColor);
    await row.hover(); await p.waitForTimeout(300);
    const c1=await row.evaluate(e=>getComputedStyle(e).backgroundColor);
    check('日志行 hover 有底色', c0!==c1, `${c0} → ${c1}`);
  }

  const img=p.locator('.survey-field-plate img, .survey-field-strip img').first();
  if (await img.count()) {
    const i0=await img.evaluate(e=>getComputedStyle(e).transform);
    await img.hover(); await p.waitForTimeout(800);
    const i1=await img.evaluate(e=>getComputedStyle(e).transform);
    check('图片 hover 缓慢推近', i0!==i1, `${i0} → ${i1}`);
  } else console.log('SKIP  无图片版面');

  // 焦点环
  await p.keyboard.press('Tab'); await p.waitForTimeout(200);
  const fo=await p.evaluate(()=>{const e=document.activeElement; return e?getComputedStyle(e).outlineColor:'none';});
  check('键盘焦点朱红', fo.includes('178, 58, 41'), fo);

  await p.screenshot({path:'/tmp/qa-home-v15-desktop.png',fullPage:false});
  await p.setViewportSize({width:390,height:844});
  await p.waitForTimeout(900);
  const of=await p.evaluate(()=>document.documentElement.scrollWidth-document.documentElement.clientWidth);
  check('手机零横向溢出', of<=0, of);
  await p.screenshot({path:'/tmp/qa-home-v15-mobile.png',fullPage:false});
  await p.context().close();
}
await b.close();
console.log(fail.length?'\nFAILED: '+fail.join(' | '):'\nALL PASS');
