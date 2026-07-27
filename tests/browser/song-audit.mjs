import { chromium } from 'playwright';
import fs from 'fs';
const BASE='https://opctoai.com/inspace';
const routes=[
 ['home','/'],['explore','/explore'],['guides','/guides'],
 ['space','/spaces/10000000-0000-0000-0000-000000000001'],
 ['login','/login'],['workspace','/host'],['admin','/admin']
];
fs.mkdirSync('output/playwright/song-audit',{recursive:true});
const b=await chromium.launch();
for (const [vpName,viewport] of [['desktop',{width:1440,height:900}],['mobile',{width:390,height:844}]]) {
 const c=await b.newContext({viewport, geolocation:{latitude:31.2397,longitude:121.4998},permissions:['geolocation']});
 await c.addCookies([{name:'instant_session',value:'qa-token-fullstack-1',domain:'opctoai.com',path:'/'}]);
 for (const [name,path] of routes) {
  const p=await c.newPage(); const errs=[];
  p.on('pageerror',e=>errs.push(e.message)); p.on('console',m=>{if(m.type()==='error') errs.push(m.text())});
  await p.goto(BASE+path,{waitUntil:'networkidle',timeout:60000}); await p.waitForTimeout(1800);
  const metrics=await p.evaluate(()=>{
   const all=[...document.querySelectorAll('main *')];
   const boxes=all.filter(e=>{const s=getComputedStyle(e);return s.borderTopWidth!=='0px'&&s.borderTopStyle!=='none';}).length;
   const shadows=all.filter(e=>getComputedStyle(e).boxShadow!=='none').length;
   const rounded=all.filter(e=>parseFloat(getComputedStyle(e).borderRadius)>=8).length;
   const paragraphs=[...document.querySelectorAll('main p')];
   return {scrollW:document.documentElement.scrollWidth,clientW:document.documentElement.clientWidth,
    boxes,shadows,rounded,paragraphs:paragraphs.length,
    text:document.querySelector('main')?.innerText.length||0};
  });
  console.log(vpName,name,JSON.stringify(metrics),'errors',errs.length);
  await p.screenshot({path:`output/playwright/song-audit/${name}-${vpName}.png`,fullPage:true});
  await p.close();
 }
 await c.close();
}
await b.close();
