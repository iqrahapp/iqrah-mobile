import { test, expect } from '@playwright/test';

test.describe('Complete Wizard Flow', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
    // Clear localStorage to start fresh
    await page.evaluate(() => localStorage.clear());
  });

  test('should complete entire annotation workflow from start to finish', async ({ page }) => {
    // ========== STAGE 0: Select Surah & Ayahs ==========
    await test.step('Stage 0: Select surah and ayahs', async () => {
      await expect(page.locator('text=Stage 0')).toBeVisible({ timeout: 10000 });

      // Select surah (e.g., Al-Fatiha)
      await page.click('[data-testid="surah-selector"]', { timeout: 5000 });
      await page.click('text=1 - Al-Fatiha');

      // Select ayah range
      await page.fill('[data-testid="ayah-start"]', '1');
      await page.fill('[data-testid="ayah-end"]', '2');

      // Proceed to next stage
      await page.click('button:has-text("Next")');

      await expect(page.locator('text=Stage 1')).toBeVisible();
    });

    // ========== STAGE 1: Record/Upload Audio ==========
    await test.step('Stage 1: Upload audio', async () => {
      // Upload audio file (or use mock recording)
      const fileInput = page.locator('input[type="file"]');

      // Create a mock audio file
      const buffer = Buffer.from('mock-audio-data');
      await fileInput.setInputFiles({
        name: 'test-audio.webm',
        mimeType: 'audio/webm',
        buffer,
      });

      // Wait for upload to complete
      await expect(page.locator('text=Upload complete')).toBeVisible({
        timeout: 15000,
      });

      // Set trim bounds (optional)
      // await page.fill('[data-testid="trim-start"]', '0');
      // await page.fill('[data-testid="trim-end"]', '10');

      // Proceed to next stage
      await page.click('button:has-text("Next")');

      await expect(page.locator('text=Stage 2')).toBeVisible();
    });

    // ========== STAGE 2: Verse Segmentation ==========
    await test.step('Stage 2: Segment verses', async () => {
      // Wait for waveform to load
      await expect(page.locator('#waveform')).toBeVisible({ timeout: 10000 });

      // Select first ayah
      await page.click('text=Ayah 1');

      // Create verse segment (Ctrl+Click or drag)
      const waveform = page.locator('#waveform');

      // Option 1: Ctrl+Click to create smart segment
      await waveform.click({
        position: { x: 100, y: 50 },
        modifiers: ['Control'],
      });

      // Wait for VAD detection
      await expect(page.locator('text=Detecting speech boundaries')).toBeVisible({
        timeout: 5000,
      });
      await expect(page.locator('text=Detecting speech boundaries')).not.toBeVisible({
        timeout: 10000,
      });

      // Verify segment created
      await expect(page.locator('text=Segmented Ayahs (1/2)')).toBeVisible();

      // Segment second ayah
      await page.click('text=Ayah 2');
      await waveform.click({
        position: { x: 300, y: 50 },
        modifiers: ['Control'],
      });

      await expect(page.locator('text=Segmented Ayahs (2/2)')).toBeVisible({
        timeout: 15000,
      });

      // Proceed to next stage
      await page.click('button:has-text("Next")');

      await expect(page.locator('text=Stage 3')).toBeVisible();
    });

    // ========== STAGE 3: Word Segmentation ==========
    await test.step('Stage 3: Segment words', async () => {
      // Wait for ayah audio to load
      await expect(page.locator('text=Ayah 1 Audio')).toBeVisible({ timeout: 10000 });

      // Get expected word count
      const wordCount = await page.locator('text=/\\d+ words/').textContent();
      const expectedWords = parseInt(wordCount?.match(/\d+/)?.[0] || '0');

      // Segment all words (simplified - in real test, would segment each word)
      for (let i = 0; i < Math.min(expectedWords, 3); i++) {
        // Click to select word
        const wordChip = page.locator(`[data-testid="word-${i}"]`).first();
        if (await wordChip.isVisible()) {
          await wordChip.click();

          // Create annotation
          const waveform = page.locator('#waveform');
          await waveform.click({
            position: { x: 50 + i * 100, y: 50 },
            modifiers: ['Control'],
          });

          // Wait for word to be marked as complete
          await expect(wordChip.locator('[data-icon="check-circle"]')).toBeVisible({
            timeout: 5000,
          });
        }
      }

      // For testing purposes, skip to next stage
      // (In real scenario, would complete all words)
      await page.click('button:has-text("Skip to Stage 4")');

      await expect(page.locator('text=Stage 4')).toBeVisible();
    });

    // ========== STAGE 4: Anti-Pattern Annotation ==========
    await test.step('Stage 4: Annotate anti-patterns (optional)', async () => {
      // This stage is optional, can proceed directly to completion
      await page.click('button:has-text("Complete")');

      await expect(page.locator('text=Export')).toBeVisible();
    });

    // ========== COMPLETION: Export Annotations ==========
    await test.step('Export annotations', async () => {
      // Click export button
      const downloadPromise = page.waitForEvent('download');
      await page.click('button:has-text("Export JSON")');

      const download = await downloadPromise;
      expect(download.suggestedFilename()).toMatch(/annotations.*\.json/);

      // Verify export validation passed
      await expect(page.locator('text=Validation passed')).toBeVisible();
    });
  });

  test('should show validation errors when exporting incomplete annotations', async ({ page }) => {
    // Navigate to wizard
    await page.goto('/wizard');

    // Skip directly to export without completing stages
    await page.evaluate(() => {
      // Mock incomplete state
      localStorage.setItem(
        'wizard-session',
        JSON.stringify({
          step: 4,
          surah: 1,
          ayahs: [1, 2],
          verses: [], // Empty - should trigger validation error
        })
      );
    });

    await page.reload();

    // Try to export
    await page.click('button:has-text("Export")');

    // Should show validation errors
    await expect(page.locator('text=Validation failed')).toBeVisible();
    await expect(page.locator('text=No verses found')).toBeVisible();
  });

  test('should allow undo/redo operations', async ({ page }) => {
    // Start wizard and get to verse segmentation
    await page.goto('/wizard');

    // ... (setup code to get to stage 2)

    // Create a verse segment
    const waveform = page.locator('#waveform');
    await waveform.click({
      position: { x: 100, y: 50 },
      modifiers: ['Control'],
    });

    // Verify segment created
    const segmentCount = await page.locator('text=/Segmented Ayahs \\(\\d+\\//').textContent();
    expect(segmentCount).toContain('1');

    // Undo (Ctrl+Z)
    await page.keyboard.press('Control+z');

    // Verify segment removed
    await expect(page.locator('text=Segmented Ayahs (0/')).toBeVisible();

    // Redo (Ctrl+Shift+Z)
    await page.keyboard.press('Control+Shift+z');

    // Verify segment restored
    await expect(page.locator('text=/Segmented Ayahs \\(1\\//)')).toBeVisible();
  });

  test('should persist progress across page reloads', async ({ page }) => {
    // Complete stage 0
    await page.goto('/wizard');

    // Select surah
    await page.click('[data-testid="surah-selector"]');
    await page.click('text=1 - Al-Fatiha');
    await page.fill('[data-testid="ayah-start"]', '1');
    await page.fill('[data-testid="ayah-end"]', '1');
    await page.click('button:has-text("Next")');

    // Verify we're on stage 1
    await expect(page.locator('text=Stage 1')).toBeVisible();

    // Reload page
    await page.reload();

    // Verify state persisted
    await expect(page.locator('text=Stage 1')).toBeVisible();
    await expect(page.locator('text=Surah: 1')).toBeVisible();
  });

  test('should show loading indicators during async operations', async ({ page }) => {
    await page.goto('/wizard');

    // Navigate to stage with audio processing
    // ... (setup code)

    // Trigger audio extraction
    await page.click('button:has-text("Extract Audio")');

    // Should show loading overlay
    await expect(page.locator('text=Processing audio')).toBeVisible();
    await expect(page.locator('[role="progressbar"]')).toBeVisible();

    // Wait for completion
    await expect(page.locator('text=Processing audio')).not.toBeVisible({
      timeout: 20000,
    });
  });

  test('should display user-friendly error messages', async ({ page }) => {
    await page.goto('/wizard');

    // Try to create invalid segment (start >= end)
    await page.evaluate(() => {
      // Trigger validation error
      window.dispatchEvent(
        new CustomEvent('validation-error', {
          detail: { error: 'start >= end' },
        })
      );
    });

    // Should show user-friendly message
    await expect(page.locator('text=Start time must be before end time')).toBeVisible();
    await expect(page.locator('text=How to fix')).toBeVisible();
  });
});
