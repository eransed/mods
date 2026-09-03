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

// test('Geometries page link is visible', async ({ page }) => {
//   await page.goto('/')
//   await expect(
//     page.getByText('Geometries')
//   ).toBeVisible({ timeout })
// })

// test('Device Connections page link is visible', async ({ page }) => {
//   await page.goto('/')
//   await expect(
//     page.getByText('Device Connections')
//   ).toBeVisible({ timeout })
// })

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

test('renders enum config properties as dropdowns and notifies after save', async ({ page }) => {
  const config = {
    general_config: {
      notification_position: { value: 'top_right', default_value: 'top_right', allowed_values: ['top_left', 'top_right', 'bottom_left', 'bottom_right'], added_version: '1.0.0', description: 'Notification position', hide: false, deprecated_version: '' },
    },
    logging_config: {
      log_level: { value: 'info', default_value: 'info', allowed_values: ['trace', 'debug', 'info', 'warn', 'error'], added_version: '1.0.0', description: 'Log level', hide: false, deprecated_version: '' },
    },
    camera_configs: [],
    open_protocol_configs: [],
    volumes: [],
  }
  await page.route('**/config', async (route) => route.fulfill({ json: config }))
  await page.route('**/default_config', async (route) => route.fulfill({ json: config }))
  await page.route('**/set_config', async (route) => route.fulfill({ json: config }))

  await page.goto('/settings')
  const position = page.locator('.config-field').filter({ hasText: 'notification_position' })
  const level = page.locator('.config-field').filter({ hasText: 'log_level' })
  await expect(position.locator('select')).toHaveValue('top_right')
  await expect(position.locator('option')).toHaveCount(4)
  await expect(position.locator('option')).toHaveText(['top_left', 'top_right', 'bottom_left', 'bottom_right'])
  await expect(position.locator('input[type="text"]')).toHaveCount(0)
  await position.locator('select').selectOption('bottom_left')
  await expect(page.locator('.notification-tray')).toHaveClass(/notification-bottom_left/)
  await expect(level.locator('select')).toHaveValue('info')
  await level.locator('select').selectOption('debug')
  await expect(page.getByRole('heading', { name: /Settings\*/ })).toBeVisible()
  await page.getByRole('button', { name: 'Save' }).click()
  await expect(page.locator('.notification-info')).toContainText('Saved 2 changes')
})

test('prompts before leaving settings with unsaved changes', async ({ page }) => {
  const config = {
    general_config: {
      http_port: { value: 8123, default_value: 8123, added_version: '1.0.0', description: 'HTTP port', hide: false, deprecated_version: '' },
      ws_port: { value: 8124, default_value: 8124, added_version: '1.0.0', description: 'Websocket port', hide: false, deprecated_version: '' },
      allow_remote_connections: { value: false, default_value: false, added_version: '1.0.0', description: 'Remote connections', hide: false, deprecated_version: '' },
    },
    logging_config: {},
    camera_configs: [],
    open_protocol_configs: [],
  }
  await page.route('**/config', async (route) => route.fulfill({ json: config }))
  await page.route('**/default_config', async (route) => route.fulfill({ json: config }))

  await page.goto('/settings')
  await page.locator('.config-field').filter({ hasText: 'http_port' }).locator('input').fill('9000')
  await page.getByRole('link', { name: 'Volumes' }).click()

  const dialog = page.getByRole('dialog')
  await expect(dialog).toBeVisible()
  await expect(dialog).toContainText('general_config.http_port')
  await dialog.getByRole('button', { name: 'Restore' }).click()
  await expect(page).toHaveURL(/\/volumes$/)
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

test('adding an OpenProtocol device marks it as just added', async ({ page, request }) => {
  const defaultResponse = await request.get('http://127.0.0.1:8123/default_config')
  const defaultConfig = await defaultResponse.json()
  const activeConfig = { ...defaultConfig, open_protocol_configs: [] }
  await page.route('**/config', async (route) => {
    await route.fulfill({ json: activeConfig })
  })

  await page.goto('/settings')
  const openProtocolSection = page.locator('section.settings-section').filter({ hasText: 'OpenProtocol Devices' }).first()
  await openProtocolSection.getByRole('button', { name: 'Add OpenProtocol Device' }).click()

  const deviceEntry = openProtocolSection.locator('h2').filter({ hasText: 'default_1' }).first()
  await expect(deviceEntry.locator('.settings-entry-name')).toHaveText('default_1')
  await expect(deviceEntry.locator('.settings-entry-connection')).toHaveText('Just added')
  await expect(deviceEntry.locator('.settings-entry-address .dot-connecting')).toBeVisible()
})

test('factory reset stages defaults until save is clicked', async ({ page, request }) => {
  const defaultResponse = await request.get('http://127.0.0.1:8123/default_config')
  const defaultConfig = await defaultResponse.json()
  const activeConfig = structuredClone(defaultConfig)
  activeConfig.general_config.ws_port.value = 9000

  let resetRequests = 0
  let saveRequests = 0
  await page.route('**/config', async (route) => {
    await route.fulfill({ json: activeConfig })
  })
  await page.route('**/reset_config', async (route) => {
    resetRequests += 1
    await route.fulfill({ status: 500, json: { error: 'reset should not be called' } })
  })
  await page.route('**/set_config', async (route) => {
    saveRequests += 1
    await route.fulfill({ json: { ok: true } })
  })

  await page.goto('/settings')
  await page.getByRole('button', { name: 'Factory reset' }).click()

  await expect(page.locator('.config-field').filter({ hasText: 'ws_port' }).locator('input')).toHaveValue(
    String(defaultConfig.general_config.ws_port.value),
  )
  await expect(page.getByRole('button', { name: 'Save' })).toBeEnabled()
  expect(resetRequests).toBe(0)
  expect(saveRequests).toBe(0)

  await page.getByRole('button', { name: 'Save' }).click()
  expect(saveRequests).toBe(1)
})

test('removing a config array entry shows the previous item count', async ({ page, request }) => {
  const defaultResponse = await request.get('http://127.0.0.1:8123/default_config')
  const defaultConfig = await defaultResponse.json()
  const cameraEntry = structuredClone(defaultConfig.camera_configs[0])
  const activeConfig = { ...defaultConfig, camera_configs: [structuredClone(cameraEntry), structuredClone(cameraEntry)] }
  activeConfig.camera_configs[0].name.value = 'remove-counter-camera-a'
  activeConfig.camera_configs[1].name.value = 'remove-counter-camera-b'

  await page.route('**/config', async (route) => {
    await route.fulfill({ json: activeConfig })
  })

  await page.goto('/settings')
  const cameraSection = page.locator('section.settings-section').filter({ hasText: 'Camera Devices' }).first()
  const heading = cameraSection.locator('h2').first()

  await expect(heading.locator('.settings-section-count')).toHaveText('(2)')
  await expect(heading.locator('.settings-section-previous-count')).toHaveCount(0)

  await cameraSection.getByRole('button', { name: 'Remove' }).first().click()
  await expect(heading.locator('.settings-section-count')).toHaveText('(1)')
  await expect(heading.locator('.settings-section-previous-count')).toHaveText('was 2')
})

test('adding config array entries shows the previous item count', async ({ page, request }) => {
  const defaultResponse = await request.get('http://127.0.0.1:8123/default_config')
  const defaultConfig = await defaultResponse.json()
  const activeConfig = { ...defaultConfig, camera_configs: [] }

  await page.route('**/config', async (route) => {
    await route.fulfill({ json: activeConfig })
  })

  await page.goto('/settings')
  const cameraSection = page.locator('section.settings-section').filter({ hasText: 'Camera Devices' }).first()
  const heading = cameraSection.locator('h2').first()

  await expect(heading.locator('.settings-section-count')).toHaveText('(0)')
  await expect(heading.locator('.settings-section-previous-count')).toHaveCount(0)

  await cameraSection.getByRole('button', { name: 'Add Camera Device' }).click()
  await expect(heading.locator('.settings-section-count')).toHaveText('(1)')
  await expect(heading.locator('.settings-section-previous-count')).toHaveText('was 0')

  await cameraSection.getByRole('button', { name: 'Add Camera Device' }).click()
  await expect(heading.locator('.settings-section-count')).toHaveText('(2)')
  await expect(heading.locator('.settings-section-previous-count')).toHaveText('was 0')
})

test('settings page does not scroll horizontally on mobile', async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 })
  const defaultConfig = {
    general_config: {
      ws_port: { value: 8124, default_value: 8124, added_version: '1.0.0', description: 'Websocket port', hide: false, deprecated_version: '' },
    },
    logging_config: {},
    camera_configs: [
      {
        name: { value: 'mobile-camera-with-a-very-long-name-that-must-stay-contained', default_value: 'camera', added_version: '1.0.0', description: 'Camera name', hide: false, deprecated_version: '' },
      },
    ],
    open_protocol_configs: [],
  }
  const activeConfig = structuredClone(defaultConfig)
  activeConfig.general_config.ws_port.value = 9000

  await page.route('**/config', async (route) => {
    await route.fulfill({ json: activeConfig })
  })
  await page.route('**/default_config', async (route) => {
    await route.fulfill({ json: defaultConfig })
  })

  await page.goto('/settings')
  await expect(page.getByRole('heading', { level: 1, name: /Settings/ })).toBeVisible()
  await page.locator('.status-stats').evaluate((statusStats) => {
    statusStats.innerHTML = '<span class="status-stat">CPU: 100.0%</span><span class="status-stat">RAM: 100.0%</span><span class="status-stat">MEM: 9999.9MB</span>'
  })
  await page.locator('.status-text').evaluate((statusText) => {
    statusText.textContent = 'connected-with-a-very-long-status-message-that-must-not-resize-the-rest-of-the-ui - 8124'
  })
  await expect(page.locator('.status-stats')).toBeVisible()

  const canScrollHorizontally = await page.evaluate(() => {
    const pageBody = document.querySelector('.page-body')
    const documentRoot = document.documentElement
    const before = {
      document: window.scrollX,
      pageBody: pageBody?.scrollLeft ?? 0,
    }

    window.scrollTo(1000, window.scrollY)
    if (pageBody) pageBody.scrollLeft = 1000

    const after = {
      document: window.scrollX,
      pageBody: pageBody?.scrollLeft ?? 0,
      documentOverflow: documentRoot.scrollWidth > documentRoot.clientWidth,
      pageBodyOverflow: pageBody ? pageBody.scrollWidth > pageBody.clientWidth : false,
    }

    window.scrollTo(before.document, window.scrollY)
    if (pageBody) pageBody.scrollLeft = before.pageBody

    return after.document > 0 || after.pageBody > 0 || after.documentOverflow || after.pageBodyOverflow
  })

  expect(canScrollHorizontally).toBe(false)

  const removeButtonBoxes = await page.getByRole('button', { name: 'Remove' }).evaluateAll((buttons) => buttons.map((button) => {
    const box = button.getBoundingClientRect()
    return { left: box.left, right: box.right }
  }))

  expect(removeButtonBoxes.length).toBeGreaterThan(0)
  for (const box of removeButtonBoxes) {
    expect(box.left).toBeGreaterThanOrEqual(0)
    expect(box.right).toBeLessThanOrEqual(390)
  }
})

test('OpenProtocol state updates the device label from a mock server', async ({ page, request }, testInfo) => {
  // This test rewrites the shared backend config, so it must not run twice in parallel.
  test.skip(testInfo.project.name !== 'chromium', 'mutates the shared backend configuration')

  const configResponse = await request.get('http://127.0.0.1:8123/config')
  const originalConfig = await configResponse.json()
  const testConfig = structuredClone(originalConfig)
  const openProtocolConfig = testConfig.open_protocol_configs[0]
  openProtocolConfig.activated.value = true
  openProtocolConfig.name.value = 'e2e-op'
  openProtocolConfig.ip.value = '127.0.0.1'
  openProtocolConfig.port.value = 5555
  openProtocolConfig.keep_alive_time_ms.value = 5

  try {
    await request.post('http://127.0.0.1:8123/set_config', { data: testConfig })
    await page.goto('/settings')

    const deviceLabel = page.locator('section.settings-section h2').filter({ hasText: 'e2e-op' }).first()
    await expect(deviceLabel.locator('.settings-entry-name')).toHaveText('e2e-op', { timeout: 15000 })
    await expect(deviceLabel.locator('.settings-entry-address-value')).toHaveText('127.0.0.1:5555')
    await expect(deviceLabel.locator('.settings-entry-connection')).toContainText(/Connected: Ping \d+ ms/, { timeout: 15000 })
  } finally {
    await request.post('http://127.0.0.1:8123/set_config', { data: originalConfig })
  }
})

