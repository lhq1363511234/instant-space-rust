import { chromium } from "playwright";
const base = process.env.BASE_URL || "https://opctoai.com";
const sizes = [[1180,820,"ipad-air-land"],[1194,834,"ipad-pro11-land"],[1366,1024,"ipad-pro129-land"],[1112,834,"ipad-pro105-land"],[1024,768,"ipad-mini-land"]];
const browser = await chromium.launch();
const out = [];
for (const [w,h,name] of sizes) {
  const ctx = await browser.newContext({ viewport:{width:w,height:h}, isMobile:false });
  const page = await ctx.newPage();
  await page.goto(`${base}/inspace`, { waitUntil:"networkidle" }).catch(()=>{});
  await page.waitForTimeout(2000);
  const info = await page.evaluate(() => {
    const box = (sel) => { const el=document.querySelector(sel); if(!el) return null; const r=el.getBoundingClientRect(); const cs=getComputedStyle(el); return {x:Math.round(r.x),y:Math.round(r.y),w:Math.round(r.width),h:Math.round(r.height),pos:cs.position,disp:cs.display,vis:cs.visibility,z:cs.zIndex}; };
    // detect elements whose text is covered by another element at its center
    const covered = [];
    document.querySelectorAll("h1,h2,h3,p,a.button,button").forEach(el=>{
      const r = el.getBoundingClientRect();
      if (r.width<10||r.height<10||r.top<0||r.top>window.innerHeight-5) return;
      const cx = r.x + Math.min(r.width/2, 60), cy = r.y + r.height/2;
      const top = document.elementFromPoint(cx, cy);
      if (top && top!==el && !el.contains(top) && !top.contains(el)) {
        covered.push({ el: el.tagName+"."+(el.className?.toString().slice(0,40)), text:(el.textContent||"").trim().slice(0,30), by: top.tagName+"."+(top.className?.toString().slice(0,40)) });
      }
    });
    return { sidebar: box(".shell-sidebar, [class*=sidebar]"), main: box(".shell-main, main"), topbar: box(".shell-topbar"), covered: covered.slice(0,6), sw: document.documentElement.scrollWidth, vw: window.innerWidth };
  });
  await page.screenshot({ path:`output/playwright/${name}.png` });
  out.push({ name, size:`${w}x${h}`, ...info });
  await ctx.close();
}
console.log(JSON.stringify(out,null,1));
await browser.close();
