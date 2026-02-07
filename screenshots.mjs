import { chromium } from 'playwright-core';

const LIB = '/tmp/clibs/extracted/usr/lib/x86_64-linux-gnu:/tmp/clibs/extracted/lib/x86_64-linux-gnu';
const DIR = '/home/node/.openclaw/workspace/parkhub/screenshots-new';

const browser = await chromium.launch({
  executablePath: '/home/node/.cache/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-linux64/chrome-headless-shell',
  args: ['--no-sandbox','--disable-setuid-sandbox','--disable-gpu'],
  env: { ...process.env, LD_LIBRARY_PATH: LIB },
});

const context = await browser.newContext({ viewport: { width: 1400, height: 900 } });
const page = await context.newPage();
page.setDefaultTimeout(10000);

async function shot(name) {
  await page.waitForTimeout(800);
  await page.screenshot({ path: `${DIR}/final-${name}.png`, fullPage: false });
  console.log(`✓ ${name}`);
}

// Login page (fresh context, no localStorage)
await page.goto('http://localhost:5173/login', { waitUntil: 'networkidle' });
await shot('login');

// Register page - use separate context to avoid token
const ctx2 = await browser.newContext({ viewport: { width: 1400, height: 900 } });
const p2 = await ctx2.newPage();
await p2.goto('http://localhost:5173/register', { waitUntil: 'networkidle' });
await p2.waitForTimeout(800);
await p2.screenshot({ path: `${DIR}/final-register.png` });
console.log('✓ register');
await p2.close();
await ctx2.close();

// Login
await page.fill('#username', 'demo');
await page.fill('#password', 'demo123');
await page.click('button[type="submit"]');
await page.waitForURL('**/', { timeout: 5000 });
await page.waitForTimeout(1500);
await shot('dashboard');

// Notifications
try {
  const headerBtns = await page.$$('header button');
  for (const btn of headerBtns) {
    const inner = await btn.innerHTML();
    if (inner.includes('Bell') || inner.includes('bell')) {
      await btn.click();
      await page.waitForTimeout(500);
      await shot('notifications');
      await btn.click();
      break;
    }
  }
} catch(e) { console.log('notif skip:', e.message); }

// Book page
await page.goto('http://localhost:5173/book', { waitUntil: 'networkidle' });
await shot('book-empty');

await page.click('text=Firmenparkplatz');
await page.waitForTimeout(1200);
await shot('book-lot-selected');

try {
  // Click an available slot - look for slot buttons with number 45
  const allBtns = await page.$$('button');
  for (const btn of allBtns) {
    const text = (await btn.textContent()).trim();
    if (text.includes('45') && !text.includes('Firmenparkplatz')) {
      await btn.click();
      break;
    }
  }
  await page.waitForTimeout(800);
  await shot('book-slot-selected');
} catch(e) { console.log('slot skip:', e.message); }

// Bookings
await page.goto('http://localhost:5173/bookings', { waitUntil: 'networkidle' });
await page.waitForTimeout(1000);
await shot('bookings');

// Vehicles
await page.goto('http://localhost:5173/vehicles', { waitUntil: 'networkidle' });
await page.waitForTimeout(500);
await shot('vehicles');

await page.click('text=Hinzufügen');
await page.waitForTimeout(600);
await shot('vehicles-add-dialog');
await page.keyboard.press('Escape');

// Homeoffice
await page.goto('http://localhost:5173/homeoffice', { waitUntil: 'networkidle' });
await page.waitForTimeout(1000);
await shot('homeoffice');

// Profile
await page.goto('http://localhost:5173/profile', { waitUntil: 'networkidle' });
await page.waitForTimeout(500);
await shot('profile');

// Admin
await page.goto('http://localhost:5173/admin', { waitUntil: 'networkidle' });
await page.waitForTimeout(500);
await shot('admin-overview');

await page.goto('http://localhost:5173/admin/lots', { waitUntil: 'networkidle' });
await page.waitForTimeout(500);
await shot('admin-lots');

try {
  await page.click('text=Bearbeiten', { timeout: 3000 });
  await page.waitForTimeout(1000);
  await shot('admin-editor');
} catch(e) { console.log('editor skip:', e.message); }

await browser.close();
console.log('\nAll done!');
