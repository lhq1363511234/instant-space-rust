import { chromium } from "playwright";
const base="https://opctoai.com", SPACE="10000000-0000-0000-0000-000000000001";
const b=await chromium.launch();
const ctx=await b.newContext({viewport:{width:1440,height:900}});
await ctx.addCookies([{name:"instant_session",value:"qa-token-fullstack-1",domain:"opctoai.com",path:"/"}]);
const p=await ctx.newPage();
const errors=[]; p.on("console",m=>{if(m.type()==="error")errors.push(m.text());});
await p.goto(`${base}/inspace/my-spaces`,{waitUntil:"networkidle"}).catch(()=>{});
await p.waitForTimeout(2500);
// open the manage modal for the seeded space
await p.screenshot({path:"output/playwright/workspace-before.png"});
const card = p.locator(".my-space-card").filter({hasText:"外滩"}).first();
const btns = await card.count();
console.log("bund card:", btns, "| all cards:", await p.locator(".my-space-card").count());
if (btns) {
  await card.locator("button", {hasText:"管理空间"}).first().click();
  await p.waitForTimeout(2000);
  await p.screenshot({path:"output/playwright/manage-modal.png"});
  const del = p.locator(".guide-list-actions button", {hasText:"删除"});
  const n = await del.count();
  console.log("delete buttons:", n);
  if (n) {
    await del.first().click();
    await p.waitForTimeout(600);
    const armed = await p.locator(".guide-list-actions button", {hasText:"确认删除"}).count();
    console.log("armed:", armed);
    await p.screenshot({path:"output/playwright/delete-armed.png"});
    if (armed) {
      await p.locator(".guide-list-actions button", {hasText:"确认删除"}).first().click();
      await p.waitForTimeout(2500);
      await p.screenshot({path:"output/playwright/delete-done.png"});
      console.log("feedback:", await p.locator(".form-success, .error").allTextContents());
    }
  }
}
console.log("errors:", errors.slice(0,4));
await b.close();
