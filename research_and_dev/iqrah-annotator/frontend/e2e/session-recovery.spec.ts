import { test, expect } from '@playwright/test';

test.describe('Session Recovery', () => {
  const STORAGE_KEY = 'wizard-session';

  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('should prompt to resume session after page reload', async ({ page }) => {
    // Create a partial annotation session
    await page.evaluate((key) => {
      localStorage.setItem(
        key,
        JSON.stringify({
          step: 2,
          surah: 1,
          ayahs: [1, 2],
          ayahTexts: [
            { ayah: 1, text: 'بِسْمِ اللَّهِ' },
            { ayah: 2, text: 'الرَّحْمَٰنِ الرَّحِيمِ' },
          ],
          recordingId: 'test-rec-123',
          audioDuration: 10.0,
          trim: { start: 0, end: 10 },
          verses: [
            {
              ayah: 1,
              start: 0,
              end: 5,
              text: 'بِسْمِ اللَّهِ',
            },
          ],
          words: [],
          antiPatterns: [],
          expectedWordCounts: {},
        })
      );
    }, STORAGE_KEY);

    // Reload page
    await page.reload();

    // Should show resume prompt
    await expect(page.locator('text=Resume previous session')).toBeVisible({
      timeout: 5000,
    });

    // Click resume
    await page.click('button:has-text("Resume")');

    // Should restore state
    await expect(page.locator('text=Stage 2')).toBeVisible();
    await expect(page.locator('text=Surah: 1')).toBeVisible();
    await expect(page.locator('text=Segmented Ayahs (1/2)')).toBeVisible();
  });

  test('should start fresh session when declining to resume', async ({ page }) => {
    // Set up saved session
    await page.evaluate((key) => {
      localStorage.setItem(
        key,
        JSON.stringify({
          step: 2,
          surah: 1,
          ayahs: [1],
        })
      );
    }, STORAGE_KEY);

    await page.reload();

    // Should show resume prompt
    await expect(page.locator('text=Resume previous session')).toBeVisible();

    // Decline resume
    await page.click('button:has-text("Start Fresh")');

    // Should be back to stage 0
    await expect(page.locator('text=Stage 0')).toBeVisible();
    await expect(page.locator('[data-testid="surah-selector"]')).toBeEmpty();
  });

  test('should auto-save every 30 seconds', async ({ page }) => {
    // Start wizard
    await page.goto('/wizard');

    // Complete stage 0
    await page.click('[data-testid="surah-selector"]');
    await page.click('text=1 - Al-Fatiha');
    await page.fill('[data-testid="ayah-start"]', '1');
    await page.fill('[data-testid="ayah-end"]', '1');

    // Wait for auto-save (should happen within 30s)
    await page.waitForTimeout(31000);

    // Check localStorage was updated
    const saved = await page.evaluate((key) => {
      return localStorage.getItem(key);
    }, STORAGE_KEY);

    expect(saved).toBeTruthy();
    const parsed = JSON.parse(saved!);
    expect(parsed.surah).toBe(1);
    expect(parsed.ayahs).toContain(1);
  });

  test('should save on page unload with unsaved changes', async ({ page, context }) => {
    await page.goto('/wizard');

    // Make some changes
    await page.click('[data-testid="surah-selector"]');
    await page.click('text=1 - Al-Fatiha');

    // Create a new page (simulates tab close)
    const newPage = await context.newPage();
    await newPage.goto('/wizard');

    // Original page should have saved on unload
    const saved = await page.evaluate((key) => {
      return localStorage.getItem(key);
    }, STORAGE_KEY);

    expect(saved).toBeTruthy();
    const parsed = JSON.parse(saved!);
    expect(parsed.surah).toBe(1);

    await newPage.close();
  });

  test('should save on tab switch', async ({ page }) => {
    await page.goto('/wizard');

    // Make changes
    await page.click('[data-testid="surah-selector"]');
    await page.click('text=1 - Al-Fatiha');
    await page.fill('[data-testid="ayah-start"]', '1');
    await page.fill('[data-testid="ayah-end"]', '2');

    // Simulate tab switch (visibility change)
    await page.evaluate(() => {
      Object.defineProperty(document, 'hidden', {
        writable: true,
        configurable: true,
        value: true,
      });

      document.dispatchEvent(new Event('visibilitychange'));
    });

    // Wait for save to complete
    await page.waitForTimeout(1000);

    // Check localStorage
    const saved = await page.evaluate((key) => {
      return localStorage.getItem(key);
    }, STORAGE_KEY);

    expect(saved).toBeTruthy();
    const parsed = JSON.parse(saved!);
    expect(parsed.ayahs).toEqual([1, 2]);
  });

  test('should show unsaved changes indicator', async ({ page }) => {
    await page.goto('/wizard');

    // Initially no unsaved changes
    await expect(page.locator('text=Unsaved changes')).not.toBeVisible();

    // Make a change
    await page.click('[data-testid="surah-selector"]');
    await page.click('text=1 - Al-Fatiha');

    // Should show unsaved changes
    await expect(page.locator('text=Unsaved changes')).toBeVisible({
      timeout: 5000,
    });

    // Wait for auto-save
    await page.waitForTimeout(31000);

    // Should clear unsaved changes indicator
    await expect(page.locator('text=Unsaved changes')).not.toBeVisible();
  });

  test('should allow manual save', async ({ page }) => {
    await page.goto('/wizard');

    // Make changes
    await page.click('[data-testid="surah-selector"]');
    await page.click('text=1 - Al-Fatiha');

    // Click save button
    await page.click('button:has-text("Save")');

    // Should show save confirmation
    await expect(page.locator('text=Saved successfully')).toBeVisible({
      timeout: 3000,
    });

    // Verify saved to localStorage
    const saved = await page.evaluate((key) => {
      return localStorage.getItem(key);
    }, STORAGE_KEY);

    expect(saved).toBeTruthy();
  });

  test('should recover from browser crash simulation', async ({ page, context }) => {
    // Setup initial session
    await page.goto('/wizard');
    await page.click('[data-testid="surah-selector"]');
    await page.click('text=1 - Al-Fatiha');
    await page.fill('[data-testid="ayah-start"]', '1');
    await page.fill('[data-testid="ayah-end"]', '3');
    await page.click('button:has-text("Next")');

    // Wait for auto-save
    await page.waitForTimeout(31000);

    // Simulate crash by closing browser without cleanup
    await context.close();

    // Create new browser context (simulates restart)
    const newContext = await page.context().browser()!.newContext();
    const newPage = await newContext.newPage();
    await newPage.goto('/wizard');

    // Should prompt to resume
    await expect(newPage.locator('text=Resume previous session')).toBeVisible({
      timeout: 5000,
    });

    await newPage.click('button:has-text("Resume")');

    // Should restore full state
    await expect(newPage.locator('text=Stage 1')).toBeVisible();
    await expect(newPage.locator('text=Surah: 1')).toBeVisible();
    await expect(newPage.locator('text=Ayahs: 1-3')).toBeVisible();

    await newContext.close();
  });

  test('should display last save timestamp', async ({ page }) => {
    await page.goto('/wizard');

    // Make a change and save
    await page.click('[data-testid="surah-selector"]');
    await page.click('text=1 - Al-Fatiha');
    await page.click('button:has-text("Save")');

    // Should show timestamp
    await expect(page.locator('text=/Last saved:.*/i')).toBeVisible({
      timeout: 3000,
    });

    // Timestamp should be recent
    const timestamp = await page.locator('text=/Last saved:.*/i').textContent();
    expect(timestamp).toMatch(/seconds? ago|just now/i);
  });

  test('should handle corrupt saved session gracefully', async ({ page }) => {
    // Set corrupt data in localStorage
    await page.evaluate((key) => {
      localStorage.setItem(key, '{invalid json}');
    }, STORAGE_KEY);

    await page.reload();

    // Should not crash, should start fresh
    await expect(page.locator('text=Stage 0')).toBeVisible();

    // Should show error notification
    await expect(page.locator('text=/Failed to load.*session/i')).toBeVisible({
      timeout: 5000,
    });
  });

  test('should clear session on explicit reset', async ({ page }) => {
    // Create session
    await page.goto('/wizard');
    await page.click('[data-testid="surah-selector"]');
    await page.click('text=1 - Al-Fatiha');

    // Wait for auto-save
    await page.waitForTimeout(31000);

    // Reset session
    await page.click('button:has-text("Reset")');
    await page.click('button:has-text("Confirm Reset")');

    // Should clear localStorage
    const saved = await page.evaluate((key) => {
      return localStorage.getItem(key);
    }, STORAGE_KEY);

    expect(saved).toBeNull();

    // Should be back to stage 0
    await expect(page.locator('text=Stage 0')).toBeVisible();
  });

  test('should preserve annotations across sessions', async ({ page }) => {
    // Create annotations
    await page.goto('/wizard');

    // ... (setup to create verse annotations)

    // Create verse annotation
    const annotation = {
      ayah: 1,
      start: 0,
      end: 5,
      text: 'بِسْمِ اللَّهِ',
    };

    await page.evaluate((ann) => {
      localStorage.setItem(
        'wizard-session',
        JSON.stringify({
          step: 2,
          verses: [ann],
        })
      );
    }, annotation);

    await page.reload();
    await page.click('button:has-text("Resume")');

    // Should restore annotations
    await expect(page.locator('text=Segmented Ayahs (1')).toBeVisible();

    // Verify annotation details
    const row = page.locator('table tbody tr').first();
    await expect(row.locator('text=1')).toBeVisible(); // Ayah number
    await expect(row.locator('text=0.000s')).toBeVisible(); // Start
    await expect(row.locator('text=5.000s')).toBeVisible(); // End
  });
});
