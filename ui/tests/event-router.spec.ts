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

test('adding a camera device uses the server default configuration', async ({ page, request }) => {
  // Use the real default endpoint while starting with an empty camera list.
  const defaultResponse = await request.get('http://127.0.0.1:8123/default_config')
  const defaultConfig = await defaultResponse.json()
  const activeConfig = { ...defaultConfig, camera_configs: [] }
  await page.route('**/config', async (route) => {
    await route.fulfill({ json: activeConfig })
  })

  await page.goto('/settings')
  const cameraSection = page.locator('section.settings-section').filter({ hasText: 'Camera Devices' }).first()
  await cameraSection.getByRole('button', { name: 'Add Camera Device' }).click()

  const cameraEntry = cameraSection.locator('section.settings-section').last()
  await expect(cameraEntry.locator('.config-field').filter({ hasText: 'name' }).locator('input')).toHaveValue(
    defaultConfig.camera_configs[0].name.value,
  )
  await expect(cameraEntry.locator('.config-field').filter({ hasText: 'device_width' }).locator('input')).toHaveValue(
    String(defaultConfig.camera_configs[0].device_width.value),
  )
  await expect(cameraEntry.locator('.config-field').filter({ hasText: 'enable_camera' }).locator('input')).toBeChecked()
})

test('OpenProtocol state updates the device label from a mock server', async ({ page, request }) => {
  const configResponse = await request.get('http://127.0.0.1:8123/config')
  const originalConfig = await configResponse.json()
  const testConfig = structuredClone(originalConfig)
  const openProtocolConfig = testConfig.open_protocol_configs[0]
  openProtocolConfig.activated.value = true
  openProtocolConfig.name.value = 'e2e-open-protocol'
  openProtocolConfig.ip.value = '127.0.0.1'
  openProtocolConfig.port.value = 5555
  openProtocolConfig.keep_alive_time_ms.value = 100

  try {
    await request.post('http://127.0.0.1:8123/set_config', { data: testConfig })
    await page.goto('/settings')

    const deviceLabel = page.locator('section.settings-section h2').filter({ hasText: 'e2e-open-protocol' }).first()
    await expect(deviceLabel).toContainText(/e2e-open-protocol - 127\.0\.0\.1:5555 - Connected: \d+ ms/)
  } finally {
    await request.post('http://127.0.0.1:8123/set_config', { data: originalConfig })
  }
})

