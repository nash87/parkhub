import { chromium } from 'playwright-core';
import { mkdirSync } from 'fs';

const LIB = '/tmp/clibs/extracted/usr/lib/x86_64-linux-gnu:/tmp/clibs/extracted/lib/x86_64-linux-gnu';
const OUT = '/home/node/.openclaw/workspace/parkhub/screenshots-all';
mkdirSync(OUT, { recursive: true });

const browser = await chromium.launch({
  executablePath: '/home/node/.cache/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-linux64/chrome-headless-shell',
  args: ['--no-sandbox','--disable-setuid-sandbox','--disable-gpu'],
  env: { ...process.env, LD_LIBRARY_PATH: LIB },
});

const ctx = await browser.newContext({ viewport: { width: 1400, height: 900 } });
const page = await ctx.newPage();
const W = ms => page.waitForTimeout(ms);
const S = async (name, opts) => { console.log(`📸 ${name}`); await page.screenshot({ path: `${OUT}/${name}`, ...opts }); };

// 1. Login page
await page.goto('http://localhost:5173/login');
await W(2000);
await S('01-login.png');

// 2. Login
const inputs = await page.$$('input');
if (inputs.length >= 2) {
  await inputs[0].fill('demo');
  await inputs[1].fill('demo');
}
await W(300);
await page.click('button[type="submit"]').catch(() => page.keyboard.press('Enter'));
await W(2500);

// 3. Dashboard full
await page.goto('http://localhost:5173/');
await W(2000);
await page.setViewportSize({ width: 1400, height: 3000 });
await W(500);
await S('02-dashboard-full.png', { fullPage: true });
await page.setViewportSize({ width: 1400, height: 900 });

// 4. Book page - select Firmenparkplatz and slot 45
await page.goto('http://localhost:5173/book');
await W(2000);
await page.click('text=Firmenparkplatz').catch(() => console.log('  Firmenparkplatz not found as clickable'));
await W(1000);
await page.click('text="45"').catch(() => console.log('  slot 45 not found'));
await W(1000);
await S('03-book-slot-selected.png');

// 5. Mehrtägig
await page.click('text=/mehrt.gig/i').catch(async () => {
  const els = await page.$$('button, label, [role="tab"], [role="radio"], select option');
  for (const e of els) {
    const t = await e.textContent().catch(() => '');
    if (/mehrt/i.test(t)) { await e.click(); break; }
  }
});
await W(1000);
await S('04-book-mehrtaegig.png');

// 6. Dauerbuchung
await page.click('text=/dauer/i').catch(async () => {
  const els = await page.$$('button, label, [role="tab"], [role="radio"]');
  for (const e of els) {
    const t = await e.textContent().catch(() => '');
    if (/dauer/i.test(t)) { await e.click(); break; }
  }
});
await W(1000);
await S('05-book-dauerbuchung.png');

// 7. Bookings
await page.goto('http://localhost:5173/bookings');
await W(2000);
await S('06-bookings.png');

// 8. Vehicles
await page.goto('http://localhost:5173/vehicles');
await W(2000);
await S('07-vehicles.png');

// 9. Homeoffice
await page.goto('http://localhost:5173/homeoffice');
await W(2000);
await S('08-homeoffice.png');

// 10. Admin Overview
await page.goto('http://localhost:5173/admin');
await W(2000);
await S('09-admin-overview.png');

// 11. Admin Parkplätze with editor
await page.goto('http://localhost:5173/admin/lots');
await W(2000);
// Try to open editor for Firmenparkplatz
await page.click('text=Firmenparkplatz').catch(() => console.log('  Could not click Firmenparkplatz in admin'));
await W(500);
await page.click('button:has-text("Bearbeiten"), [aria-label*="edit" i], button:has-text("Edit")').catch(() => console.log('  No edit button found'));
await W(1000);
await S('10-admin-lots-editor.png');

// 12. Admin Benutzer
await page.goto('http://localhost:5173/admin/users');
await W(2000);
await S('11-admin-users.png');

// 13. Admin Buchungen
await page.goto('http://localhost:5173/admin/bookings');
await W(2000);
await S('12-admin-bookings.png');

// 14. Profile
await page.goto('http://localhost:5173/profile');
await W(2000);
await S('13-profile.png');

// 15. Dashboard dark mode
await page.goto('http://localhost:5173/');
await W(2000);
// Find and click dark mode toggle (moon icon)
const clicked = await page.evaluate(() => {
  const btns = document.querySelectorAll('button');
  for (const b of btns) {
    const svg = b.querySelector('svg');
    if (svg && (b.innerHTML.includes('Moon') || b.innerHTML.includes('moon') || b.getAttribute('aria-label')?.includes('dark') || b.getAttribute('aria-label')?.includes('theme'))) {
      b.click(); return true;
    }
  }
  // Try any button with moon-like icon
  for (const b of btns) {
    if (b.innerHTML.includes('oon') || b.innerHTML.includes('heme') || b.innerHTML.includes('dark')) {
      b.click(); return true;
    }
  }
  return false;
});
console.log('  Dark mode toggle clicked:', clicked);
await W(1000);
await S('14-dashboard-dark.png');

// 16. Dashboard English
const langClicked = await page.evaluate(() => {
  const els = document.querySelectorAll('button, a, span');
  for (const e of els) {
    const t = e.textContent?.trim();
    if (t === 'EN' || t === 'English') { e.click(); return true; }
  }
  // Try select
  const sel = document.querySelector('select');
  if (sel) {
    for (const opt of sel.options) {
      if (opt.value === 'en' || opt.text.includes('EN') || opt.text.includes('English')) {
        sel.value = opt.value;
        sel.dispatchEvent(new Event('change', { bubbles: true }));
        return true;
      }
    }
  }
  return false;
});
console.log('  Language toggle clicked:', langClicked);
await W(1500);
await S('15-dashboard-english.png');

await browser.close();
console.log('✅ All screenshots saved to', OUT);
