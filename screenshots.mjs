import { chromium } from 'playwright-core';

const LIB = '/tmp/clibs/extracted/usr/lib/x86_64-linux-gnu:/tmp/clibs/extracted/lib/x86_64-linux-gnu';
const browser = await chromium.launch({
  executablePath: '/home/node/.cache/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-linux64/chrome-headless-shell',
  args: ['--no-sandbox','--disable-setuid-sandbox','--disable-gpu'],
  env: { ...process.env, LD_LIBRARY_PATH: LIB },
});

const dir = '/home/node/.openclaw/workspace/parkhub/screenshots-all';

async function login(page) {
  await page.goto('http://localhost:5173/login');
  await page.fill('#username', 'demo');
  await page.fill('#password', 'demo');
  await page.click('button[type="submit"]');
  await page.waitForURL('http://localhost:5173/');
  await page.waitForTimeout(1500);
}

// Mobile screenshots
const mobileCtx = await browser.newContext({ viewport: { width: 375, height: 812 } });
const mp = await mobileCtx.newPage();
await login(mp);

const mobilePages = [
  ['/', 'mobile-dashboard'],
  ['/book', 'mobile-book'],
  ['/bookings', 'mobile-bookings'],
  ['/homeoffice', 'mobile-homeoffice'],
];

for (const [url, name] of mobilePages) {
  await mp.goto(`http://localhost:5173${url}`);
  await mp.waitForTimeout(1500);
  await mp.screenshot({ path: `${dir}/${name}.png`, fullPage: true });
  console.log(`✓ ${name}`);
}
await mobileCtx.close();

// Desktop screenshots
const deskCtx = await browser.newContext({ viewport: { width: 1440, height: 900 } });
const dp = await deskCtx.newPage();
await login(dp);

const desktopPages = [
  ['/', 'fix-dashboard'],
  ['/homeoffice', 'fix-homeoffice'],
];

for (const [url, name] of desktopPages) {
  await dp.goto(`http://localhost:5173${url}`);
  await dp.waitForTimeout(1500);
  await dp.screenshot({ path: `${dir}/${name}.png`, fullPage: true });
  console.log(`✓ ${name}`);
}
await deskCtx.close();

await browser.close();
console.log('Done!');
