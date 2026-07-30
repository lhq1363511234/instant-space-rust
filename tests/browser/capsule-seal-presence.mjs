import { chromium } from 'playwright';

const BASE='https://opctoai.com/inspace';
const SPACE='10000000-0000-0000-0000-000000000001';
const CODE='481902';
const browser=await chromium.launch();
const failures=[];
const check=(name,ok,detail='')=>{console.log(ok?'PASS':'FAIL',name,detail); if(!ok)failures.push({name,detail});};

async function prepare(context, recipient) {
  await context.addCookies([{name:'instant_session',value:'qa-token-fullstack-1',domain:'opctoai.com',path:'/'}]);
  const page=await context.newPage();
  const errors=[];
  page.on('pageerror',e=>errors.push(e.message));
  page.on('console',m=>{if(m.type()==='error') errors.push(m.text())});
  await page.goto(`${BASE}/spaces/${SPACE}`,{waitUntil:'networkidle',timeout:60000});
  await page.locator('.capsule-shelf-head button').click();
  const fields=page.locator('.capsule-composer input[type=text]');
  await fields.nth(0).fill(recipient);
  await page.locator('.capsule-composer textarea').fill('只有在现场才能埋下的测试信。');
  await fields.nth(1).fill('现场双验证');
  return {page,errors,submit:page.locator('.capsule-composer button[type=submit]')};
}

// Near the Bund: neither proof, code only, then both proofs.
{
  const context=await browser.newContext({viewport:{width:1280,height:1000},geolocation:{latitude:31.2397,longitude:121.4998},permissions:['geolocation']});
  const {page,errors,submit}=await prepare(context,'埋写双验证 '+Date.now());
  check('无证明时不能提交',await submit.isDisabled());
  await page.locator('.presence-code input').fill(CODE);
  await page.locator('.presence-code button[type=submit]').click();
  await page.waitForTimeout(1500);
  check('只有 Wi-Fi 口令仍不能提交',await submit.isDisabled());
  await page.locator('.capsule-bury-gate button:has-text("验证当前位置")').click();
  await page.waitForTimeout(1800);
  check('Wi-Fi + 本地定位后可以提交',!(await submit.isDisabled()));
  await submit.click();
  await page.waitForTimeout(2500);
  check('双验证后胶囊写入成功',await page.locator('.capsule-composer').count()===0);
  check('近场流程无浏览器错误',errors.length===0,errors.slice(0,2).join(' | '));
  await context.close();
}

// Beijing coordinates with the correct Wi-Fi code: client may submit, server must reject distance.
{
  const context=await browser.newContext({viewport:{width:390,height:844},geolocation:{latitude:39.9042,longitude:116.4074},permissions:['geolocation']});
  const {page,errors,submit}=await prepare(context,'远程埋写拒绝 '+Date.now());
  await page.locator('.presence-code input').fill(CODE);
  await page.locator('.presence-code button[type=submit]').click();
  await page.waitForTimeout(1400);
  await page.locator('.capsule-bury-gate button:has-text("验证当前位置")').click();
  await page.waitForTimeout(1700);
  await submit.click();
  await page.waitForTimeout(2200);
  const message=(await page.locator('.capsule-composer .form-error').innerText().catch(()=>''));
  check('正确 Wi-Fi 口令但位置太远仍被服务端拒绝',message.includes('距离这里约'),message);
  check('手机没有横向溢出',(await page.evaluate(()=>document.documentElement.scrollWidth-document.documentElement.clientWidth))<=0);
  check('远场流程无浏览器错误',errors.length===0,errors.slice(0,2).join(' | '));
  await context.close();
}

await browser.close();
if(failures.length){console.log(JSON.stringify(failures,null,2));process.exit(1)}
console.log('ALL PASS');
