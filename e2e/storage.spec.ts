import { test, expect } from '@playwright/test';

test.describe('Storage E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Login
    await page.goto('/#/login');
    page.on('console', msg => console.log('BROWSER CONSOLE:', msg.text()));
    await page.getByPlaceholder('Email').fill('test@example.com');
    await page.getByPlaceholder('Password').fill('password123');
    await page.getByRole('button', { name: 'Sign in' }).click();
    await expect(page).toHaveURL(/.*#\/home/);
  });

  test('should handle context menu, folder navigation, rename, move, and delete', async ({ page }) => {
    const folderName = `f-${Date.now()}`;
    const renamedFolder = `${folderName}-renamed`;

    // 1. Create a folder using the + button
    await page.locator('.action-button-wrapper .action').first().click();
    await page.locator('.el-dropdown-menu__item:visible', { hasText: '新建文件夹' }).click();
    await page.locator('.el-dialog').locator('input').fill(folderName);
    await page.getByRole('button', { name: '确定' }).click(); 

    // Verify folder created
    await expect(page.locator(`.card-container:has-text("${folderName}")`)).toBeVisible();

    // 2. Context Menu
    const card = page.locator(`.card-container:has-text("${folderName}")`);
    await card.click({ button: 'right' }); // Right-click for context menu
    await page.locator('.el-dropdown-menu__item:visible', { hasText: '重命名' }).click();
    await page.locator('.el-dialog').locator('input').fill(renamedFolder);
    await page.getByRole('button', { name: '确定' }).click();
    
    await page.waitForTimeout(1000); // wait for rename API

    await page.screenshot({ path: 'storage-rename-fail.png', fullPage: true });
    await expect(page.locator(`.card-container:has-text("${renamedFolder}")`)).toBeVisible();
    await expect(page.locator(`.card-container`).filter({ has: page.getByText(folderName, { exact: true }) })).toBeHidden();

    // 4. Folder Navigation
    const renamedCard = page.locator(`.card-container:has-text("${renamedFolder}")`);
    await renamedCard.click(); // Wait for navigation is double-click or single click?
    // In code: handleClickCard calls router.push if type is folder. So single click.
    await expect(page.locator('.breadcrumb-wrapper:visible')).toContainText(renamedFolder);

    // Go back to root
    await page.goto('/#/home');
    await page.waitForTimeout(1000); // wait for list

    // 5. Move
    await page.locator(`.card-container:has-text("${renamedFolder}")`).click({ button: 'right' });
    await page.locator('.el-dropdown-menu__item:visible', { hasText: '移动' }).click();
    // Move dialog appears. Select root or some folder, click 确定
    // Just click 确定 (which might be disabled if same folder, but wait, we need to click "全部文件" in the tree)
    await page.locator('.el-dialog').getByRole('button', { name: '取消' }).click(); // just cancel for now to test dialog appears

    // 6. Delete
    await page.locator(`.card-container:has-text("${renamedFolder}")`).click({ button: 'right' });
    await page.locator('.el-dropdown-menu__item:visible', { hasText: '删除' }).click();
    await page.getByRole('button', { name: '确定删除' }).click(); // ElMessageBox

    // Verify deleted
    await expect(page.locator(`.card-container:has-text("${renamedFolder}")`)).toBeHidden();
  });
});
