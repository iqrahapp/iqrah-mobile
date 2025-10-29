#!/usr/bin/env node

/**
 * Simple UI test to check for errors
 */

import { chromium } from '@playwright/test';

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();

  const errors = [];
  const logs = [];

  // Capture console errors
  page.on('console', msg => {
    const type = msg.type();
    const text = msg.text();
    logs.push(`[${type}] ${text}`);
    if (type === 'error') {
      errors.push(text);
    }
  });

  // Capture page errors
  page.on('pageerror', error => {
    errors.push(`Page Error: ${error.message}`);
  });

  console.log('Opening http://localhost:5173...');
  await page.goto('http://localhost:5173', { waitUntil: 'networkidle' });

  // Wait a bit for everything to load
  await page.waitForTimeout(2000);

  console.log('\n=== Console Logs ===');
  logs.forEach(log => console.log(log));

  if (errors.length > 0) {
    console.log('\n=== ERRORS FOUND ===');
    errors.forEach(err => console.error(err));
    process.exit(1);
  } else {
    console.log('\n✅ No errors found!');

    // Check if app rendered
    const title = await page.title();
    console.log(`Page title: ${title}`);

    const appBar = await page.locator('header').count();
    console.log(`App bar found: ${appBar > 0 ? 'Yes' : 'No'}`);

    process.exit(0);
  }

  await browser.close();
})();
