import { chromium } from 'playwright-core';

const LIB = '/tmp/clibs/extracted/usr/lib/x86_64-linux-gnu:/tmp/clibs/extracted/lib/x86_64-linux-gnu';
const browser = await chromium.launch({
  executablePath: '/home/node/.cache/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-linux64/chrome-headless-shell',
  args: ['--no-sandbox','--disable-setuid-sandbox','--disable-gpu'],
  env: { ...process.env, LD_LIBRARY_PATH: LIB },
});

const BASE = 'http://172.18.0.3:5173';
const DIR = '/home/node/.openclaw/workspace/parkhub/screenshots-new';

async function shot(page, name) {
  await page.waitForTimeout(1000);
  await page.screenshot({ path: `${DIR}/v2-${name}.png` });
  console.log(`✓ ${name}`);
}

// ── Login screenshot ──
const loginCtx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
const lp = await loginCtx.newPage();
await lp.goto(BASE + '/login');
await lp.waitForTimeout(5000);
await shot(lp, 'login');
await loginCtx.close();

// ── Authenticated context ──
const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
const page = await ctx.newPage();
await page.goto(BASE + '/login');
await page.waitForTimeout(4000);
await page.fill('#username', 'admin');
await page.fill('#password', 'test');
await page.click('button[type="submit"]');
await page.waitForTimeout(3000);

// Dashboard dark mode
await page.evaluate(() => {
  document.documentElement.classList.add('dark');
});
await page.waitForTimeout(500);
await shot(page, 'dashboard-dark');

// Book page dark
await page.goto(BASE + '/book');
await page.waitForTimeout(2000);
await shot(page, 'book-dark');

// Light mode
await page.evaluate(() => {
  document.documentElement.classList.remove('dark');
});

// Admin overview
await page.goto(BASE + '/admin');
await page.waitForTimeout(3000);
await shot(page, 'admin-overview');

// Admin users
await page.goto(BASE + '/admin/users');
await page.waitForTimeout(2000);
await shot(page, 'admin-users');

// Admin bookings
await page.goto(BASE + '/admin/bookings');
await page.waitForTimeout(2000);
await shot(page, 'admin-bookings');

// Booking success modal
await page.goto(BASE + '/book');
await page.waitForTimeout(2000);
try {
  // Click first lot card
  const lotCard = page.locator('button').filter({ hasText: 'frei' }).first();
  await lotCard.click();
  await page.waitForTimeout(2500);
  
  // Click available slot
  const slot = page.locator('.slot-available').first();
  await slot.click();
  await page.waitForTimeout(1500);
  
  // Enter plate
  const plateInput = page.locator('input[placeholder*="Kennzeichen"]');
  if (await plateInput.count() > 0) await plateInput.fill('M-AB 1234');
  await page.waitForTimeout(500);
  
  // Book
  const bookBtn = page.locator('button').filter({ hasText: 'Jetzt buchen' });
  await bookBtn.click();
  await page.waitForTimeout(2500);
  await shot(page, 'booking-success');
} catch(e) {
  console.log('Booking flow error:', e.message);
}

// Mobile view
const mc = await browser.newContext({ viewport: { width: 390, height: 844 } });
const mp = await mc.newPage();
await mp.goto(BASE + '/login');
await mp.waitForTimeout(4000);
await mp.fill('#username', 'admin');
await mp.fill('#password', 'test');
await mp.click('button[type="submit"]');
await mp.waitForTimeout(3000);
await shot(mp, 'mobile-dashboard');

await browser.close();
console.log('All done!');
