import { expect, test, type Page } from '@playwright/test'

const volume = {
  name: 'sphere_1',
  position: { x: 0, y: 0, z: 0 },
  enter_radius: 5,
  exit_radius: 7.5,
  coordinate_system: 'world',
}

const config = {
  general_config: {
    http_port: { value: 8123, default_value: 8123, added_version: '1.0.0', description: 'Http port', hide: false, deprecated_version: '' },
    ws_port: { value: 8124, default_value: 8124, added_version: '1.0.0', description: 'Websocket port', hide: false, deprecated_version: '' },
    allow_remote_connections: { value: false, default_value: false, added_version: '1.0.0', description: 'Allow remote connections', hide: false, deprecated_version: '' },
  },
  logging_config: {},
  camera_configs: [],
  open_protocol_configs: [],
  volumes: [volume],
}

/** Serves a single sphere volume so the 3D view always renders a known, centered target. */
async function mockConfig(page: Page) {
  await page.route('**/set_config', async (route) => {
    await route.fulfill({ json: { status: 'ok' } })
  })
  await page.route('**/config', async (route) => {
    await route.fulfill({ json: config })
  })
}

/** Clicks the middle of the 3D view, where the configured sphere is framed. */
async function clickSphere(page: Page) {
  const box = await page.getByTestId('view-container').boundingBox()
  if (!box) throw new Error('view container has no layout')
  await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2)
  return box
}

/** Clicks a corner of the 3D view, which is empty space around the sphere. */
async function clickEmptySpace(page: Page) {
  const box = await page.getByTestId('view-container').boundingBox()
  if (!box) throw new Error('view container has no layout')
  await page.mouse.click(box.x + 5, box.y + 5)
}

test('selecting a sphere opens the editor panel with its data', async ({ page }) => {
  await mockConfig(page)
  await page.goto('/view')

  await expect(page.getByTestId('volume-panel')).toBeHidden()
  await clickSphere(page)

  await expect(page.getByTestId('volume-panel')).toBeVisible()
  await expect(page.getByTestId('volume-panel-title')).toHaveText('sphere_1')
  await expect(page.getByTestId('volume-panel-enter-radius')).toHaveValue('5')
  await expect(page.getByTestId('volume-panel-exit-radius')).toHaveValue('7.5')
})

test('the close cross hides the editor panel', async ({ page }) => {
  await mockConfig(page)
  await page.goto('/view')
  await clickSphere(page)

  const panel = page.getByTestId('volume-panel')
  await expect(panel).toBeVisible()
  await page.getByTestId('volume-panel-close').click()
  await expect(panel).toBeHidden()
})

test('clicking empty space deselects the sphere and closes the panel', async ({ page }) => {
  await mockConfig(page)
  await page.goto('/view')
  await clickSphere(page)

  const panel = page.getByTestId('volume-panel')
  await expect(panel).toBeVisible()
  await clickEmptySpace(page)
  await expect(panel).toBeHidden()
})

test('sphere data can be edited and saved from the panel', async ({ page }) => {
  await mockConfig(page)
  let savedBody: string | null = null
  await page.route('**/set_config', async (route) => {
    savedBody = route.request().postData()
    await route.fulfill({ json: { status: 'ok' } })
  })

  await page.goto('/view')
  await clickSphere(page)
  await expect(page.getByTestId('volume-panel')).toBeVisible()

  await page.getByTestId('volume-panel-name').fill('sphere_edited')
  await page.getByTestId('volume-panel-x').fill('1.5')
  await page.getByTestId('volume-panel-enter-radius').fill('6')
  await expect(page.getByTestId('volume-panel-title')).toHaveText('sphere_edited')

  await page.getByTestId('volume-panel-save').click()

  await expect.poll(() => savedBody).not.toBeNull()
  const saved = JSON.parse(savedBody ?? '{}')
  expect(saved.volumes[0]).toMatchObject({
    name: 'sphere_edited',
    position: { x: 1.5, y: 0, z: 0 },
    enter_radius: 6,
    exit_radius: 7.5,
  })
})

test('the panel sits beside the 3D view on wide screens', async ({ page }) => {
  await mockConfig(page)
  await page.setViewportSize({ width: 1280, height: 800 })
  await page.goto('/view')
  await clickSphere(page)

  const view = await page.getByTestId('view-container').boundingBox()
  const panel = await page.getByTestId('volume-panel').boundingBox()
  if (!view || !panel) throw new Error('missing layout')
  // Sidebar on the right of the 3D view.
  expect(panel.x).toBeGreaterThanOrEqual(view.x + view.width - 1)
})

test('the panel fills the bottom half on small screens', async ({ page }) => {
  await mockConfig(page)
  await page.setViewportSize({ width: 420, height: 800 })
  await page.goto('/view')
  await clickSphere(page)

  const view = await page.getByTestId('view-container').boundingBox()
  const panel = await page.getByTestId('volume-panel').boundingBox()
  if (!view || !panel) throw new Error('missing layout')
  // Stacked: the panel starts where the 3D view ends and covers roughly the same height.
  expect(panel.y).toBeGreaterThanOrEqual(view.y + view.height - 2)
  expect(Math.abs(panel.height - view.height)).toBeLessThan(4)
})
