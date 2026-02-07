import { chromium } from 'playwright-core';

const SCREENSHOTS_DIR = '/home/node/.openclaw/workspace/parkhub/screenshots-new';
const LIB_PATH = '/tmp/clibs/extracted/usr/lib/x86_64-linux-gnu:/tmp/clibs/extracted/lib/x86_64-linux-gnu';

async function main() {
  const browser = await chromium.launch({
    executablePath: '/home/node/.cache/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-linux64/chrome-headless-shell',
    args: ['--no-sandbox', '--disable-setuid-sandbox', '--disable-gpu'],
    env: { ...process.env, LD_LIBRARY_PATH: LIB_PATH },
  });
  
  const context = await browser.newContext({
    viewport: { width: 1280, height: 900 },
    colorScheme: 'light',
  });
  
  const page = await context.newPage();
  
  // Login page
  await page.goto('http://localhost:5173/login');
  await page.waitForTimeout(2000);
  await page.screenshot({ path: `${SCREENSHOTS_DIR}/01-login.png`, fullPage: true });
  console.log('✅ Login page');
  
  // The app redirects to login if not authenticated, so dashboard shows login
  // We need to mock auth or just show public pages + the grid component standalone
  
  // Register page
  await page.goto('http://localhost:5173/register');
  await page.waitForTimeout(1500);
  await page.screenshot({ path: `${SCREENSHOTS_DIR}/02-register.png`, fullPage: true });
  console.log('✅ Register page');
  
  // Dark mode login
  const darkContext = await browser.newContext({
    viewport: { width: 1280, height: 900 },
    colorScheme: 'dark',
  });
  const darkPage = await darkContext.newPage();
  await darkPage.goto('http://localhost:5173/login');
  await darkPage.waitForTimeout(2000);
  await darkPage.screenshot({ path: `${SCREENSHOTS_DIR}/03-login-dark.png`, fullPage: true });
  console.log('✅ Login dark mode');

  // Dashboard with mock auth - inject token to bypass auth
  const dashContext = await browser.newContext({
    viewport: { width: 1400, height: 1000 },
    colorScheme: 'light',
  });
  const dashPage = await dashContext.newPage();
  // Set mock auth data in localStorage before navigating
  await dashPage.goto('http://localhost:5173/login');
  await dashPage.waitForTimeout(500);
  await dashPage.evaluate(() => {
    localStorage.setItem('parkhub_token', 'mock-token-for-screenshots');
  });
  await dashPage.goto('http://localhost:5173/');
  await dashPage.waitForTimeout(2500);
  await dashPage.screenshot({ path: `${SCREENSHOTS_DIR}/04-dashboard.png`, fullPage: true });
  console.log('✅ Dashboard');

  // Admin page
  await dashPage.goto('http://localhost:5173/admin/lots');
  await dashPage.waitForTimeout(2000);
  await dashPage.screenshot({ path: `${SCREENSHOTS_DIR}/05-admin-lots.png`, fullPage: true });
  console.log('✅ Admin Lots');
  
  // Book page
  await dashPage.goto('http://localhost:5173/book');
  await dashPage.waitForTimeout(2000);
  await dashPage.screenshot({ path: `${SCREENSHOTS_DIR}/06-book.png`, fullPage: true });
  console.log('✅ Book');

  // Dark dashboard
  const darkDashContext = await browser.newContext({
    viewport: { width: 1400, height: 1000 },
    colorScheme: 'dark',
  });
  const darkDashPage = await darkDashContext.newPage();
  await darkDashPage.goto('http://localhost:5173/login');
  await darkDashPage.waitForTimeout(500);
  await darkDashPage.evaluate(() => {
    localStorage.setItem('parkhub_token', 'mock-token-for-screenshots');
    localStorage.setItem('parkhub-theme', JSON.stringify({ state: { isDark: true }, version: 0 }));
  });
  await darkDashPage.goto('http://localhost:5173/');
  await darkDashPage.waitForTimeout(2500);
  await darkDashPage.screenshot({ path: `${SCREENSHOTS_DIR}/07-dashboard-dark.png`, fullPage: true });
  console.log('✅ Dashboard dark');
  
  await browser.close();
  console.log('Done!');
}

main().catch(console.error);
