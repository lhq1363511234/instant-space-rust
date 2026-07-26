import { chromium } from "playwright";
const b = await chromium.launch();
for (const [w,h,name] of [[1360,900,"desk"],[390,844,"phone"]]) {
  const p = await (await b.newContext({viewport:{width:w,height:h}})).newPage();
  await p.goto("https://opctoai.com/inspace/guides",{waitUntil:"networkidle"}).catch(()=>{});
  await p.waitForTimeout(4000);
  const over = await p.evaluate(()=>({ sw: document.documentElement.scrollWidth, vw: innerWidth,
    rowH: document.querySelector(".guide-list > li")?.getBoundingClientRect().height ?? null,
    rows: document.querySelectorAll(".guide-list > li").length }));
  console.log(name, JSON.stringify(over));
  await p.evaluate(()=>window.scrollTo(0, 620));
  await p.waitForTimeout(600);
  await p.screenshot({path:`/tmp/v-guides-${name}.png`});
}
await b.close();
