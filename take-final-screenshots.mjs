import { chromium } from 'playwright-core';
import { mkdirSync } from 'fs';

const LIB = '/tmp/clibs/extracted/usr/lib/x86_64-linux-gnu:/tmp/clibs/extracted/lib/x86_64-linux-gnu';
const DIR = '/home/node/.openclaw/workspace/parkhub/screenshots-new';
mkdirSync(DIR, { recursive: true });

const browser = await chromium.launch({
  executablePath: '/home/node/.cache/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-linux64/chrome-headless-shell',
  args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
  env: { ...process.env, LD_LIBRARY_PATH: LIB },
});

const context = await browser.newContext({ viewport: { width: 1400, height: 900 } });
const page = await context.newPage();

// Login
await page.goto('http://localhost:5173/login');
await page.waitForTimeout(1000);
await page.fill('input[type="text"], input[name="username"], input:first-of-type', 'demo');
await page.fill('input[type="password"]', 'demo');
await page.click('button[type="submit"]');
await page.waitForTimeout(2000);

// 1. Dashboard
await page.goto('http://localhost:5173/');
await page.waitForTimeout(2000);
await page.screenshot({ path: `${DIR}/01-dashboard.png`, fullPage: true });
console.log('✓ Dashboard');

// 2. Book page - select lot and see grid
await page.goto('http://localhost:5173/book');
await page.waitForTimeout(1500);
// Click on first lot
const lotButtons = await page.$$('button');
for (const btn of lotButtons) {
  const text = await btn.textContent();
  if (text?.includes('Firmenparkplatz')) { await btn.click(); break; }
}
await page.waitForTimeout(1500);
await page.screenshot({ path: `${DIR}/02-book-lot-grid.png`, fullPage: true });
console.log('✓ Book page with grid');

// Click an available slot
const slotButtons = await page.$$('button:not([disabled])');
for (const btn of slotButtons) {
  const text = await btn.textContent();
  if (text?.match(/^4[5-9]|^5[0-7]/) && !text?.includes('M-')) {
    await btn.click();
    break;
  }
}
await page.waitForTimeout(1000);
await page.screenshot({ path: `${DIR}/03-book-slot-selected.png`, fullPage: true });
console.log('✓ Book page with slot selected');

// Switch to Mehrtägig
const mehrtaegigBtns = await page.$$('button');
for (const btn of mehrtaegigBtns) {
  const text = await btn.textContent();
  if (text?.includes('Mehrtägig')) { await btn.click(); break; }
}
await page.waitForTimeout(500);
await page.screenshot({ path: `${DIR}/04-book-mehrtaegig.png`, fullPage: true });
console.log('✓ Book multi-day');

// Switch to Dauerbuchung
const dauerBtns = await page.$$('button');
for (const btn of dauerBtns) {
  const text = await btn.textContent();
  if (text?.includes('Dauerbuchung')) { await btn.click(); break; }
}
await page.waitForTimeout(500);
await page.screenshot({ path: `${DIR}/05-book-dauerbuchung.png`, fullPage: true });
console.log('✓ Book permanent');

// 3. Bookings page
await page.goto('http://localhost:5173/bookings');
await page.waitForTimeout(1500);
await page.screenshot({ path: `${DIR}/06-bookings.png`, fullPage: true });
console.log('✓ Bookings');

// 4. Vehicles page
await page.goto('http://localhost:5173/vehicles');
await page.waitForTimeout(1500);
await page.screenshot({ path: `${DIR}/07-vehicles.png`, fullPage: true });
console.log('✓ Vehicles');

// 5. Admin lots with editor
await page.goto('http://localhost:5173/admin/lots');
await page.waitForTimeout(1500);
// Click Bearbeiten on first lot
const editBtns = await page.$$('button');
for (const btn of editBtns) {
  const text = await btn.textContent();
  if (text?.includes('Bearbeiten')) { await btn.click(); break; }
}
await page.waitForTimeout(1500);
await page.screenshot({ path: `${DIR}/08-admin-lots-editor.png`, fullPage: true });
console.log('✓ Admin lots with editor');

// 6. User menu dropdown
await page.goto('http://localhost:5173/');
await page.waitForTimeout(1000);
const userMenuBtns = await page.$$('button');
for (const btn of userMenuBtns) {
  const text = await btn.textContent();
  if (text?.includes('Max')) { await btn.click(); break; }
}
await page.waitForTimeout(500);
await page.screenshot({ path: `${DIR}/09-user-menu.png` });
console.log('✓ User menu');

await browser.close();
console.log('\n✅ All screenshots saved to', DIR);
