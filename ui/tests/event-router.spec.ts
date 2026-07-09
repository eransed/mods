import { expect, test } from '@playwright/test'

test('renders the EventRouter page', async ({ page }) => {
  await page.goto('/')

  await expect(
    page.getByRole('heading', { level: 1, name: 'EventRouter' }),
  ).toBeVisible()
  await expect(page.getByText(/connecting|connected|disconnected/i)).toBeVisible()
})

test('Overview page link is visible', async ({ page }) => {
  await page.goto('/')
  await expect(
    page.getByText('Overview')
  ).toBeVisible({ timeout: 1000 })
})

test('Volumes page link is visible', async ({ page }) => {
  await page.goto('/')
  await expect(
    page.getByText('Volumes')
  ).toBeVisible({ timeout: 1000 })
})

test('Geometries page link is visible', async ({ page }) => {
  await page.goto('/')
  await expect(
    page.getByText('Geometries')
  ).toBeVisible({ timeout: 1000 })
})

test('Device Connections page link is visible', async ({ page }) => {
  await page.goto('/')
  await expect(
    page.getByText('Device Connections')
  ).toBeVisible({ timeout: 1000 })
})
