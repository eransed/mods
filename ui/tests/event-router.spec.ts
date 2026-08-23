import { expect, test } from '@playwright/test'

const timeout = 2000

test('renders the mods main page', async ({ page }) => {
  await page.goto('/')

  await expect(
    page.getByRole('heading', { level: 1, name: 'mods' }),
  ).toBeVisible()
  await expect(page.getByText(/connecting|connected|disconnected/i)).toBeVisible()
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

test('About page link is visible', async ({ page }) => {
  await page.goto('/')
  await expect(
    page.getByText('About')
  ).toBeVisible({ timeout })
})

