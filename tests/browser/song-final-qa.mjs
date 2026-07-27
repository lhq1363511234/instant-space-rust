import { chromium } from 'playwright';
import fs from 'fs';
const BASE='https://opctoai.com/inspace';
const routes=[
 ['home','/'],['explore','/explore'],['guides','/guides'],
 ['space','/spaces/10000000-0000-0000-0000-000000000001'],
 ['login','/login'],['workspace','/my-spaces'],['admin','/admin']
];
const viewports=[['desktop',{width:1440,height:900}],['ipad-landscape',{width:1024,height:768}],['ipad-portrait',{width:768,height:1024}],['phone',{width:390,height:844}],['small-phone',{width:375,height:812}]];
fs.mkdirSync('output/playwright/song-final',{recursive:true});
const b=await chromium.launch(); let failures=[];
for(const [vpName,viewport] of viewports){
 const c=await b.newContext({viewport,geolocation:{latitude:31.2397,longitude:121.4998},permissions:['geolocation']});
 await c.addCookies([{name:'instant_session',value:'qa-token-fullstack-1',domain:'opctoai.com',path:'/'}]);
 for(const [name,path] of routes){
  if(!['desktop','phone'].includes(vpName) && !['home','guides','space','login'].includes(name)) continue;
  const p=await c.newPage(); const errs=[]; const failed=[];
  p.on('pageerror',e=>errs.push(e.message));
  p.on('console',m=>{if(m.type()==='error') errs.push(m.text())});
  p.on('requestfailed',r=>failed.push(`${r.method()} ${r.url()} ${r.failure()?.errorText}`));
  await p.goto(BASE+path,{waitUntil:'networkidle',timeout:60000}); await p.waitForTimeout(1000);
  const r=await p.evaluate(()=>{
   const main=document.querySelector('main');
   const buttons=[...document.querySelectorAll('main button, main a.button')].filter(e=>getComputedStyle(e).display!=='none');
   const smallTargets=buttons.filter(e=>{const x=e.getBoundingClientRect(); return x.width<44||x.height<44}).map(e=>(e.textContent||e.getAttribute('aria-label')||'').trim()).slice(0,5);
   const labels=[...document.querySelectorAll('main input,main select,main textarea')].filter(e=>e.type!=='hidden').filter(e=>!e.labels?.length&&!e.getAttribute('aria-label')).length;
   const h1=document.querySelector('main h1'); const hr=h1?.getBoundingClientRect();
   const nav=document.querySelector('.shell-mobile-nav'); const nr=nav?.getBoundingClientRect();
   const submit=document.querySelector('.auth-form button[type=submit]'); const sr=submit?.getBoundingClientRect();
   return {
    overflow:document.documentElement.scrollWidth-document.documentElement.clientWidth,
    h1Clip:!!hr&&(hr.left<0||hr.right>innerWidth+1),
    smallTargets,unlabelled:labels,
    mainText:(main?.innerText||'').length,
    authCovered:!!(nr&&sr&&sr.bottom>nr.top&&sr.top<nr.bottom),
    focusColor:getComputedStyle(document.documentElement).getPropertyValue('--song-focus').trim()
   };
  });
  const ok=r.overflow<=0&&!r.h1Clip&&r.unlabelled===0&&!r.authCovered&&errs.length===0&&failed.filter(x=>!x.includes('favicon')).length===0;
  console.log(ok?'PASS':'FAIL',vpName,name,JSON.stringify(r),'errs',errs.length,'reqfail',failed.length);
  if(!ok)failures.push({vpName,name,r,errs,failed});
  if(['desktop','phone'].includes(vpName)) await p.screenshot({path:`output/playwright/song-final/${name}-${vpName}.png`,fullPage:true});
  await p.close();
 }
 await c.close();
}
// Reduced motion must remove authored entrances.
{
 const c=await b.newContext({viewport:{width:390,height:844},reducedMotion:'reduce'}); const p=await c.newPage();
 await p.goto(BASE,{waitUntil:'networkidle'}); await p.waitForTimeout(500);
 const an=await p.locator('.survey-hero h1').evaluate(e=>getComputedStyle(e).animationName);
 const ok=an==='none'; console.log(ok?'PASS':'FAIL','reduced-motion',an); if(!ok)failures.push({reducedMotion:an});
 await c.close();
}
await b.close();
console.log(failures.length?`FAILED ${failures.length}`:'ALL PASS');
if(failures.length) console.log(JSON.stringify(failures,null,2));
