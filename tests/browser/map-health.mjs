import { chromium } from "playwright";
const b = await chromium.launch();
const p = await (await b.newContext({viewport:{width:1360,height:900}})).newPage();
const fails=[], tiles={ok:0,bad:0};
p.on("response", r => {
  const u=r.url(), s=r.status();
  if (/tile|\.pbf|\.png|openfreemap|maptiler/i.test(u)) { s<400?tiles.ok++:tiles.bad++; }
  if (s>=400) fails.push(s+" "+u.slice(0,110));
});
p.on("console", m => { if (m.type()==="error") fails.push("console: "+m.text().slice(0,110)); });
await p.goto("https://opctoai.com/inspace/map",{waitUntil:"networkidle"}).catch(()=>{});
await p.waitForTimeout(9000);
const st = await p.evaluate(()=>{
  const s=globalThis.__instantSpaceMaps?.get?.("map");
  const c=document.querySelector("#map canvas");
  return { hasMap:!!s?.map, styleLoaded:s?.map?.isStyleLoaded?.(), zoom:s?.map?.getZoom?.(),
    canvas: c?{w:c.width,h:c.height}:null, clusters:document.querySelectorAll(".map-cluster").length,
    loadingVisible: getComputedStyle(document.querySelector(".map-loading")||document.body).display };
});
console.log(JSON.stringify({st, tiles, fails:fails.slice(0,8)},null,2));
await p.screenshot({path:"/tmp/map-health.png"});
await b.close();
