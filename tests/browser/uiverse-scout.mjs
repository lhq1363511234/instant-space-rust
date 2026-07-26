import { chromium } from 'playwright';
const b = await chromium.launch();
const c = await b.newContext({viewport:{width:1440,height:1200},
  userAgent:'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36'});
const p = await c.newPage();
for (const cat of ['inputs','loaders','checkboxes']) {
  try {
    await p.goto(`https://uiverse.io/${cat}`, {waitUntil:'domcontentloaded', timeout:45000});
    await p.waitForTimeout(3500);
    await p.screenshot({path:`/tmp/uiverse-${cat}.png`});
    console.log(cat, 'ok');
  } catch(e){ console.log(cat,'ERR',e.message.slice(0,60)); }
}
await b.close();
