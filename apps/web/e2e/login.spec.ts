import { expect, test } from '@playwright/test'

test.describe('Login E2E', () => {
  test('should render login page', async ({ page }) => {
    await page.goto('/#/login')
    await expect(page.getByRole('heading', { name: 'Log in' })).toBeVisible()
    await expect(page.getByPlaceholder('Email')).toBeVisible()
    await expect(page.getByPlaceholder('Password')).toBeVisible()
    await expect(page.getByRole('button', { name: 'Sign in' })).toBeVisible()
  })

  test('should fail on incorrect credentials with toast message', async ({
    page,
  }) => {
    await page.goto('/#/login')
    await page.getByPlaceholder('Email').fill('wrong@example.com')
    await page.getByPlaceholder('Password').fill('wrongpass')
    await page.getByRole('button', { name: 'Sign in' }).click()

    const errorToast = page.locator('.el-message--error')
    await expect(errorToast).toBeVisible()
  })

  test('should succeed on valid credentials and navigate to Home', async ({
    page,
  }) => {
    await page.goto('/login')
    await page.getByPlaceholder('Email').fill('test@example.com')
    await page.getByPlaceholder('Password').fill('password123')
    await page.getByRole('button', { name: 'Sign in' }).click()

    await expect(page).toHaveURL(/.*#\/home/)
  })

  test('should maintain session persistence across page reloads', async ({
    page,
  }) => {
    await page.goto('/#/login')
    await page.getByPlaceholder('Email').fill('test@example.com')
    await page.getByPlaceholder('Password').fill('password123')
    await page.getByRole('button', { name: 'Sign in' }).click()
    await expect(page).toHaveURL(/.*#\/home/)

    await page.reload()
    await expect(page).toHaveURL(/.*#\/home/)
  })
})
