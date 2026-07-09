import { expect, test } from '@playwright/test'

const timeout = 2000

test('renders the EventRouter page', async ({ page }) => {
  await page.goto('/')

  await expect(
    page.getByRole('heading', { level: 1, name: 'EventRouter' }),
  ).toBeVisible()
  await expect(page.getByText(/connecting|connected|disconnected/i)).toBeVisible()
})

test('Menu is visible', async ({ page }) => {
  await page.goto('/')
  await expect(
    page.getByText('Menu')
  ).toBeVisible({ timeout })
})

test('Volumes page link is visible', async ({ page }) => {
  await page.goto('/')
  await expect(
    page.getByText('Volumes')
  ).toBeVisible({ timeout })
})

test('Geometries page link is visible', async ({ page }) => {
  await page.goto('/')
  await expect(
    page.getByText('Geometries')
  ).toBeVisible({ timeout })
})

test('Device Connections page link is visible', async ({ page }) => {
  await page.goto('/')
  await expect(
    page.getByText('Device Connections')
  ).toBeVisible({ timeout })
})
