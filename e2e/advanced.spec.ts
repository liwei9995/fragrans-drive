import * as fs from 'node:fs'
import * as path from 'node:path'
import { expect, test } from '@playwright/test'

test.describe('Advanced E2E', () => {
  test.beforeEach(async ({ page }) => {
    // Login before tests
    await page.goto('/#/login')
    page.on('console', (msg) => console.log('BROWSER CONSOLE:', msg.text()))
    await page.getByPlaceholder('Email').fill('test@example.com')
    await page.getByPlaceholder('Password').fill('password123')
    await page.getByRole('button', { name: 'Sign in' }).click()
    await expect(page).toHaveURL(/.*#\/home/, { timeout: 15000 })
  })

  test('Create Folder', async ({ page }) => {
    const folderName = `adv-folder-${Date.now()}`

    // Create a folder using the + button
    await page.locator('.action-button-wrapper .action').first().click()
    await page
      .locator('.el-dropdown-menu__item:visible', { hasText: '新建文件夹' })
      .click()
    await page.locator('.el-dialog').locator('input').fill(folderName)
    await page.getByRole('button', { name: '确定' }).click()

    // Verify folder created
    await expect(
      page.locator(`.card-container:has-text("${folderName}")`),
    ).toBeVisible()
  })

  test('Batch Actions', async ({ page }) => {
    // Create 2 folders to batch delete
    const folder1 = `batch1-${Date.now()}`
    const folder2 = `batch2-${Date.now()}`

    // Create folder 1
    await page.locator('.action-button-wrapper .action').first().click()
    await page
      .locator('.el-dropdown-menu__item:visible', { hasText: '新建文件夹' })
      .click()
    await page.locator('.el-dialog').locator('input').fill(folder1)
    await page.getByRole('button', { name: '确定' }).click()
    await expect(
      page.locator(`.card-container:has-text("${folder1}")`),
    ).toBeVisible()

    // Create folder 2
    await page.locator('.action-button-wrapper .action').first().click()
    await page
      .locator('.el-dropdown-menu__item:visible', { hasText: '新建文件夹' })
      .click()
    await page.locator('.el-dialog').locator('input').fill(folder2)
    await page.getByRole('button', { name: '确定' }).click()
    await expect(
      page.locator(`.card-container:has-text("${folder2}")`),
    ).toBeVisible()

    // Select both folders via checkbox
    const card1 = page.locator(`.card-container:has-text("${folder1}")`)
    await card1.hover()
    await card1
      .locator('.selection-checkbox')
      .evaluate((el) => (el as HTMLElement).click())

    const card2 = page.locator(`.card-container:has-text("${folder2}")`)
    await card2.hover()
    await card2
      .locator('.selection-checkbox')
      .evaluate((el) => (el as HTMLElement).click())

    // Verify Floating action bar is visible
    const actionBar = page.locator('.floating-action-bar-wrapper')
    await expect(actionBar).toBeVisible()
    await expect(actionBar.locator('.count')).toHaveText('2')

    // Click delete
    await actionBar.locator('.action-item.delete').click()

    // Confirm delete in modal
    await page.getByRole('button', { name: '确定删除' }).click()

    // Verify both folders are deleted
    await expect(
      page.locator(`.card-container:has-text("${folder1}")`),
    ).toBeHidden()
    await expect(
      page.locator(`.card-container:has-text("${folder2}")`),
    ).toBeHidden()
  })

  test('Download', async ({ page }) => {
    // Upload a mock file
    const testFileName = `download-test-${Date.now()}.txt`

    await page
      .locator('input[type="file"]')
      .first()
      .setInputFiles({
        name: testFileName,
        mimeType: 'text/plain',
        buffer: Buffer.from(`download content ${Date.now()}`),
      })

    // wait for upload success
    const uploadStatus = page.locator('.upload-status-wrapper')
    await expect(uploadStatus).toBeVisible()
    await expect(page.locator('.upload-status .success')).toBeVisible({
      timeout: 10000,
    })
    await expect(uploadStatus).toBeHidden({ timeout: 5000 })

    await page.waitForTimeout(2000) // Wait for backend processing
    await page.reload()
    await page.waitForTimeout(2000) // Wait for page to load

    await expect(
      page.locator(`.card-container:has-text("${testFileName}")`),
    ).toBeVisible()

    // Trigger download
    const card = page.locator(`.card-container:has-text("${testFileName}")`)
    await card.click({ button: 'right' }) // Right-click for context menu

    // Start waiting for download before clicking
    const downloadPromise = page.waitForEvent('download')
    await page
      .locator('.el-dropdown-menu__item:visible', { hasText: '下载' })
      .click()

    const download = await downloadPromise
    expect(download.suggestedFilename()).toBe(testFileName)
  })

  test('Media Preview', async ({ page }) => {
    // Upload an image file
    const testImageName = `preview-test-${Date.now()}.svg`

    // Create a valid, unique SVG image so it has a unique hash and renders correctly in the browser
    const svgContent = `<svg xmlns="http://www.w3.org/2000/svg" width="100" height="100"><text x="10" y="50">${Date.now()}</text></svg>`
    const imageBuffer = Buffer.from(svgContent)

    await page.locator('input[type="file"]').first().setInputFiles({
      name: testImageName,
      mimeType: 'image/svg+xml',
      buffer: imageBuffer,
    })

    const uploadStatus = page.locator('.upload-status-wrapper')
    await expect(uploadStatus).toBeVisible()
    await expect(page.locator('.upload-status .success')).toBeVisible({
      timeout: 15000,
    })
    await expect(uploadStatus).toBeHidden({ timeout: 15000 })

    await page.waitForTimeout(4000) // Wait for backend processing
    await page.reload()
    await page.waitForLoadState('domcontentloaded')
    await page.waitForTimeout(3000) // Wait for page to load

    const card = page.locator(`.card-container:has-text("${testImageName}")`)
    await expect(card).toBeVisible({ timeout: 15000 })

    // Click on the cover image to trigger preview
    await card.locator('.cover').click()

    // Verify preview modal appears
    await expect(page.locator('.el-image-viewer__wrapper')).toBeVisible()

    // Close the preview
    await page.locator('.el-image-viewer__close').click()
    await expect(page.locator('.el-image-viewer__wrapper')).toBeHidden()
  })
})
