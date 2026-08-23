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

test('settings checkbox changes when the custom checkbox is clicked', async ({ page }) => {
  // Provide the smallest Config-shaped response needed to render a checkbox.
  await page.route('**/config', async (route) => {
    await route.fulfill({
      json: {
        general_config: {
          ws_port: { value: 8124, default_value: 8124, added_version: '1.0.0', description: 'Websocket port', hide: false, deprecated_version: '' },
          allow_remote_connections: { value: false, default_value: false, added_version: '1.0.0', description: 'Allow remote connections', hide: false, deprecated_version: '' },
        },
        logging_config: {},
        camera_configs: [],
        open_protocol_configs: [],
      },
    })
  })

  await page.goto('/settings')
  const row = page.locator('.config-field').filter({ hasText: 'allow_remote_connections' })
  const checkbox = row.locator('input[type="checkbox"]')

  await expect(checkbox).not.toBeChecked()
  await row.locator('.checkbox').click()
  await expect(checkbox).toBeChecked()
})

