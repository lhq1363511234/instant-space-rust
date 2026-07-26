// Presence claims a visitor can simply assert must not survive contradicting
// evidence. `?via=qr` is a string anyone can append to a URL; if the browser
// also volunteers coordinates a thousand kilometres away, the scan is a lie.
import { chromium } from 'playwright';

const SPACE='10000000-0000-0000-0000-000000000001';
const EP='https://opctoai.com/inspace/api/leave_trace16784949679025184826';
const FAR={lat:39.9042,lng:116.4074};   // Beijing
const NEAR={lat:31.2397,lng:121.4998};  // the Bund itself

const b=await chromium.launch();
const c=await b.newContext();
await c.addCookies([{name:'instant_session',value:'qa-token-fullstack-1',domain:'opctoai.com',path:'/'}]);
const p=await c.newPage();
await p.goto('https://opctoai.com/inspace',{waitUntil:'domcontentloaded'});

async function leave(label, {scanned, lat, lng, code}) {
  const proof = await p.evaluate(async ({EP,scanned,lat,lng,code}) => {
    const f=new URLSearchParams();
    f.set('space_id','10000000-0000-0000-0000-000000000001');
    f.set('body','forgery probe '+Date.now());
    f.set('scanned',String(scanned));
    if(lat!=null){f.set('lat',String(lat));f.set('lng',String(lng));}
    f.set('discord_member','false');
    if(code!=null)f.set('onsite_code',code);
    const r=await fetch(EP,{method:'POST',body:f,credentials:'include'});
    const t=await r.text();
    try{return JSON.parse(t).proof}catch{return t.slice(0,60)}
  },{EP,scanned,lat,lng,code});
  console.log(`${label.padEnd(46)} -> ${proof}`);
  return proof;
}

const results = {};
results.forgedScanFar   = await leave('forged scanned=true, 1000km away', {scanned:true, ...FAR});
results.forgedScanNoGeo = await leave('forged scanned=true, no coords', {scanned:true});
results.realScanNear    = await leave('scanned=true, actually at the place', {scanned:true, ...NEAR});
results.wrongCodeFar    = await leave('wrong code, 1000km away', {scanned:false, ...FAR, code:'000000'});
results.rightCodeFar    = await leave('correct code, 1000km away', {scanned:false, ...FAR, code:'481902'});
results.noneAtAll       = await leave('nothing offered', {scanned:false});

console.log('\n--- expectations ---');
const check=(name,actual,expected)=>console.log(`${actual===expected?'PASS':'FAIL'}  ${name}: ${actual} (want ${expected})`);
check('forged scan + far coords must NOT be scan', results.forgedScanFar, 'remote');
check('forged scan with no coords still honoured', results.forgedScanNoGeo, 'scan');
check('genuine scan at the place', results.realScanNear, 'scan');
check('wrong code is remote', results.wrongCodeFar, 'remote');
check('correct code beats distance', results.rightCodeFar, 'on_site');
check('nothing offered is remote', results.noneAtAll, 'remote');
await b.close();
