import { chromium } from "playwright";
const b = await chromium.launch();
const p = await (await b.newContext({viewport:{width:1360,height:900}})).newPage();
const errors=[]; p.on("console", m=>{ if(m.type()==="error") errors.push(m.text()); });
await p.goto("https://opctoai.com/inspace/map",{waitUntil:"networkidle"}).catch(()=>{});
await p.waitForTimeout(7000);
const trail=[];
for (let i=0;i<6;i++){
  const st = await p.evaluate(()=>{
    const s=globalThis.__instantSpaceMaps.get("map");
    return { zoom:+s.map.getZoom().toFixed(2), clusters:document.querySelectorAll(".map-cluster").length, pins:document.querySelectorAll(".map-marker").length, libre:document.querySelectorAll(".maplibregl-marker").length };
  });
  trail.push(st);
  if (st.clusters===0) break;
  await p.evaluate(()=>{
    // click the cluster nearest the viewport centre so we keep drilling into view
    const cx=innerWidth/2, cy=innerHeight/2;
    let best=null,bd=Infinity;
    for (const el of document.querySelectorAll(".map-cluster")){
      const r=el.getBoundingClientRect();
      const d=Math.hypot(r.x+r.width/2-cx, r.y+r.height/2-cy);
      if(d<bd){bd=d;best=el;}
    }
    best?.click();
  });
  await p.waitForTimeout(2200);
}
// now open a pin
let drawerOpened=false, pinName=null;
const hasPin = await p.locator(".map-marker").count();
if (hasPin>0){
  pinName = await p.locator(".map-marker-label").first().textContent().catch(()=>null);
  await p.evaluate(()=>{ document.querySelector(".map-marker")?.click(); });
  await p.waitForTimeout(2200);
  drawerOpened = (await p.locator(".space-detail-drawer").count())>0;
}
await p.screenshot({path:"/tmp/v-drill-final.png"});
console.log(JSON.stringify({trail, hasPin, pinName, drawerOpened, errors:errors.slice(0,5)},null,2));
await b.close();
