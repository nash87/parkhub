import { chromium } from 'playwright-core';
const DIR = '/home/node/.openclaw/workspace/parkhub/screenshots-new';
const LIB = '/tmp/clibs/extracted/usr/lib/x86_64-linux-gnu:/tmp/clibs/extracted/lib/x86_64-linux-gnu';

const browser = await chromium.launch({
  executablePath: '/home/node/.cache/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-linux64/chrome-headless-shell',
  args: ['--no-sandbox','--disable-setuid-sandbox','--disable-gpu'],
  env: { ...process.env, LD_LIBRARY_PATH: LIB },
});

// Light mode - login first to get token
const ctx = await browser.newContext({ viewport: { width: 1400, height: 1000 }, colorScheme: 'light' });
const p = await ctx.newPage();
await p.goto('http://localhost:5173/login');
await p.waitForTimeout(1500);
await p.fill('#username', 'demo');
await p.fill('#password', 'demo123');
await p.click('button[type="submit"]');
await p.waitForTimeout(2000);
await p.screenshot({ path: `${DIR}/01-dashboard.png`, fullPage: true });
console.log('✅ Dashboard');

await p.goto('http://localhost:5173/book');
await p.waitForTimeout(2000);
await p.screenshot({ path: `${DIR}/02-book.png`, fullPage: true });
console.log('✅ Book');

await p.goto('http://localhost:5173/bookings');
await p.waitForTimeout(1500);
await p.screenshot({ path: `${DIR}/03-bookings.png`, fullPage: true });
console.log('✅ Bookings');

await p.goto('http://localhost:5173/vehicles');
await p.waitForTimeout(1500);
await p.screenshot({ path: `${DIR}/04-vehicles.png`, fullPage: true });
console.log('✅ Vehicles');

await p.goto('http://localhost:5173/admin');
await p.waitForTimeout(1500);
await p.screenshot({ path: `${DIR}/05-admin.png`, fullPage: true });
console.log('✅ Admin');

await p.goto('http://localhost:5173/admin/lots');
await p.waitForTimeout(1500);
await p.screenshot({ path: `${DIR}/06-admin-lots.png`, fullPage: true });
console.log('✅ Admin Lots');

// Dark mode
const dark = await browser.newContext({ viewport: { width: 1400, height: 1000 }, colorScheme: 'dark' });
const dp = await dark.newPage();
await dp.goto('http://localhost:5173/login');
await dp.waitForTimeout(500);
await dp.fill('#username', 'demo');
await dp.fill('#password', 'demo123');
await dp.click('button[type="submit"]');
await dp.waitForTimeout(2000);
// toggle dark mode
await dp.evaluate(() => {
  document.documentElement.classList.add('dark');
  localStorage.setItem('parkhub-theme', JSON.stringify({ state: { isDark: true }, version: 0 }));
});
await dp.waitForTimeout(500);
await dp.reload();
await dp.waitForTimeout(2000);
await dp.screenshot({ path: `${DIR}/07-dashboard-dark.png`, fullPage: true });
console.log('✅ Dashboard Dark');

await browser.close();
console.log('Done!');
