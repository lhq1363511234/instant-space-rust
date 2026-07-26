import { chromium } from 'playwright';
const fail=[]; const check=(n,ok,got)=>{console.log((ok?'PASS  ':'FAIL  ')+n+(ok?'':`  got=${got}`)); if(!ok)fail.push(n);};
const b=await chromium.launch();
const p=await (await b.newContext({viewport:{width:1280,height:900}})).newPage();
const errs=[]; p.on('pageerror',e=>errs.push(e.message));
p.on('console',m=>{if(m.type()==='error')errs.push('console: '+m.text())});
await p.goto('https://opctoai.com/inspace/login',{waitUntil:'networkidle'});
await p.waitForTimeout(2500);

const i=p.locator('.auth-form input').first();
const st=await i.evaluate(e=>{const s=getComputedStyle(e);return{r:s.borderRadius,t:s.borderTopWidth,b:s.borderBottomWidth};});
check('输入框无圆角', st.r==='0px', st.r);
check('输入框只有底线', st.t==='0px' && st.b==='1px', JSON.stringify(st));

await i.click(); await p.waitForTimeout(400);
const line=await p.locator('.auth-form .field-label').first().evaluate(e=>getComputedStyle(e,'::after').transform);
check('聚焦朱红底线展开', /matrix\(1,/.test(line), line);
const lc=await p.locator('.auth-form .field-label > span').first().evaluate(e=>getComputedStyle(e).color);
check('聚焦标签转朱红', lc.includes('178, 58, 41'), lc);

const tab=p.locator('.auth-tabs button').first();
const tb=await tab.evaluate(e=>getComputedStyle(e).backgroundColor);
check('Tab 非蓝色药丸', tb==='rgba(0, 0, 0, 0)', tb);
const ind=await tab.evaluate(e=>getComputedStyle(e,'::after').transform);
check('选中 Tab 有朱红指示线', /matrix\(1,/.test(ind), ind);

const corner=await p.locator('.auth-form').evaluate(e=>getComputedStyle(e,'::before').backgroundColor);
check('登录卡有朱红角标', corner.includes('178, 58, 41'), corner);
const an=await p.locator('.auth-form').evaluate(e=>getComputedStyle(e).animationName);
check('表单有入场动效', an!=='none', an);

await p.screenshot({path:'/tmp/qa-login-v15b.png',fullPage:true});
await p.setViewportSize({width:390,height:844}); await p.waitForTimeout(800);
check('手机零横向溢出', await p.evaluate(()=>document.documentElement.scrollWidth-document.documentElement.clientWidth)<=0,'');
await p.screenshot({path:'/tmp/qa-login-v15b-mobile.png',fullPage:true});
check('无 JS 错误', errs.length===0, errs.slice(0,2));
await b.close();
console.log(fail.length?'\nFAILED: '+fail.join(' | '):'\nALL PASS');
