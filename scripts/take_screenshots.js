const { chromium, firefox } = require('playwright');

(async () => {
  let browser;
  try {
    browser = await chromium.launch({ headless: true });
    console.log('Using Chromium');
  } catch (e) {
    console.log('Chromium failed, trying Firefox...', e.message);
    try {
      browser = await firefox.launch({ headless: true });
      console.log('Using Firefox');
    } catch (e2) {
      console.error('Both browsers failed:', e2.message);
      process.exit(1);
    }
  }

  const context = await browser.newContext({
    viewport: { width: 1280, height: 800 },
    colorScheme: 'dark'
  });
  const page = await context.newPage();

  // Login page
  await page.goto('http://localhost:7878/login');
  await page.waitForTimeout(2000);
  await page.screenshot({ path: 'docs/screenshots/login.png' });
  console.log('✓ login.png');

  // Login
  const usernameInput = page.locator('input').first();
  await usernameInput.fill('admin');
  const passwordInput = page.locator('input[type="password"]');
  await passwordInput.fill('Test1234!');
  await page.locator('button[type="submit"]').click();
  await page.waitForTimeout(2000);

  // Dashboard
  await page.screenshot({ path: 'docs/screenshots/dashboard.png' });
  console.log('✓ dashboard.png');

  // Booking page
  await page.goto('http://localhost:7878/book');
  await page.waitForTimeout(2000);
  await page.screenshot({ path: 'docs/screenshots/booking.png' });
  console.log('✓ booking.png');

  // Admin page
  await page.goto('http://localhost:7878/admin');
  await page.waitForTimeout(2000);
  await page.screenshot({ path: 'docs/screenshots/admin.png' });
  console.log('✓ admin.png');

  // Settings/themes
  await page.goto('http://localhost:7878/settings');
  await page.waitForTimeout(1500);
  await page.screenshot({ path: 'docs/screenshots/themes.png' });
  console.log('✓ themes.png');

  // Welcome page (unauthenticated view)
  const context2 = await browser.newContext({
    viewport: { width: 1280, height: 800 },
    colorScheme: 'dark'
  });
  const page2 = await context2.newPage();
  await page2.goto('http://localhost:7878/welcome');
  await page2.waitForTimeout(2000);
  await page2.screenshot({ path: 'docs/screenshots/welcome.png' });
  console.log('✓ welcome.png');
  await context2.close();

  // Mobile view
  await page.setViewportSize({ width: 375, height: 812 });
  await page.goto('http://localhost:7878/');
  await page.waitForTimeout(2000);
  await page.screenshot({ path: 'docs/screenshots/mobile.png' });
  console.log('✓ mobile.png');

  // Dark mode booking (mobile)
  await page.goto('http://localhost:7878/book');
  await page.waitForTimeout(2000);
  await page.screenshot({ path: 'docs/screenshots/dark-mode.png' });
  console.log('✓ dark-mode.png');

  await browser.close();
  console.log('\nAll screenshots captured!');
})();
