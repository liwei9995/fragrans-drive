import { test, expect } from '@playwright/test';

test.describe('Upload E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Login before tests
    await page.goto('/#/login');
    page.on('console', msg => console.log('BROWSER CONSOLE:', msg.text()));
    await page.getByPlaceholder('Email').fill('test@example.com');
    await page.getByPlaceholder('Password').fill('password123');
    await page.getByRole('button', { name: 'Sign in' }).click();
    await expect(page).toHaveURL(/.*#\/home/);
  });

  test('should upload a file and display status popup', async ({ page }) => {
    // We can use Playwright's setInputFiles to upload a file to the hidden input.
    const testFileName = `test-upload-${Date.now()}.txt`;
    
    await page.locator('input[type="file"]').first().setInputFiles({
      name: testFileName,
      mimeType: 'text/plain',
      buffer: Buffer.from(`this is a test file contents ${Date.now()}`)
    });

    // The upload status popup should appear
    const uploadStatus = page.locator('.upload-status-wrapper');
    await expect(uploadStatus).toBeVisible();
    
    // Check that it shows success (Wait for the success icon or title to reflect success)
    await expect(page.locator('.upload-status .success')).toBeVisible({ timeout: 10000 });

    // Verify it automatically closes after 3 seconds (max 5000ms wait)
    await expect(uploadStatus).toBeHidden({ timeout: 5000 });

    await page.waitForTimeout(2000); // Wait for backend processing
    await page.reload();
    await page.waitForTimeout(2000); // Wait for page to load

    await page.screenshot({ path: 'upload-fail.png', fullPage: true });
    await expect(page.locator(`text=${testFileName}`)).toBeVisible();
  });
});
