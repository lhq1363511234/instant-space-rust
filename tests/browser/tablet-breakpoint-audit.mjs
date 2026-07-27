import { chromium } from 'playwright';
import fs from 'fs';
fs.mkdirSync('output/playwright/tablet-breakpoint',{recursive:true});
const b=await chromium.launch();
const widths=[1024,1100,1140,1180,1194,1240,1280,1320,1366,1440];
const routes=[['home','/'],['explore','/explore'],['guides','/guides'],['space','/spaces/10000000-0000-0000-0000-000000000001'],['login','/login'],['workspace','/my-spaces'],['admin','/admin']];
for(const width of widths){
 const c=await b.newContext({viewport:{width,height:900},isMobile:false,hasTouch:true,deviceScaleFactor:2});
 await c.addCookies([{name:'instant_session',value:'qa-token-fullstack-1',domain:'opctoai.com',path:'/'}]);
 for(const [name,path] of routes){
  const p=await c.newPage();
  await p.goto('https://opctoai.com/inspace'+path,{waitUntil:'networkidle'});
  await p.waitForTimeout(400);
  const r=await p.evaluate(()=>{
   const app=document.querySelector('.app-main')?.getBoundingClientRect();
   const headings=[...document.querySelectorAll('main h1, main h2')].map((h,i)=>{
    const s=getComputedStyle(h), r=h.getBoundingClientRect(), line=parseFloat(s.lineHeight);
    return {i,tag:h.tagName,cls:h.className||'',text:(h.textContent||'').trim().replace(/\s+/g,' ').slice(0,60),w:Math.round(r.width),h:Math.round(r.height),font:+parseFloat(s.fontSize).toFixed(1),lines:line?Math.round(r.height/line):0};
   }).filter(x=>x.w>0&&x.h>0);
   const badHeadings=headings.filter(x=>x.lines>=5);
   const grids=[...document.querySelectorAll('.survey-hero,.survey-passage,.survey-field-head,.survey-keep-head,.auth-page,.space-detail-layout,.admin-layout')].map(el=>{const s=getComputedStyle(el),r=el.getBoundingClientRect();return {cls:el.className,w:Math.round(r.width),cols:s.gridTemplateColumns,gap:s.columnGap};});
   return {appW:Math.round(app?.width||0),overflow:document.documentElement.scrollWidth-innerWidth,badHeadings,headings,grids};
  });
  const bad=r.overflow>0||r.badHeadings.length>0;
  console.log(bad?'BAD ':'OK  ',width,name,JSON.stringify({appW:r.appW,overflow:r.overflow,bad:r.badHeadings,grids:r.grids}));
  if(bad) await p.screenshot({path:`output/playwright/tablet-breakpoint/${name}-${width}.png`,fullPage:true});
  await p.close();
 }
 await c.close();
}
await b.close();
