import { useEffect, useRef, useState, type CSSProperties, type PointerEvent as ReactPointerEvent } from 'react'
import * as THREE from 'three'
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js'
import type { Config, Volume } from '../types/Config'

export interface ViewProps {
  port: number
}

/** Width below which the editor moves from a right sidebar to the bottom half of the screen. */
const narrowWidthPx = 600

/** Pointer travel in pixels that still counts as a click instead of an orbit drag. */
const clickSlopPx = 5

const rowStyle: CSSProperties = {
  display: 'flex',
  alignItems: 'center',
  justifyContent: 'space-between',
  gap: '0.5rem',
  marginTop: '0.4rem',
}

const inputStyle: CSSProperties = { width: '8rem' }

/** 3D view rendering the volumes configured in the backend config. */
export function View({ port }: ViewProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const groupRef = useRef<THREE.Group | null>(null)
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null)
  const controlsRef = useRef<OrbitControls | null>(null)
  const axesRef = useRef<THREE.AxesHelper | null>(null)
  const pickablesRef = useRef<THREE.Mesh[]>([])
  const raycasterRef = useRef(new THREE.Raycaster())
  const pointerDownRef = useRef<{ x: number; y: number } | null>(null)
  const framedRef = useRef(false)
  const [config, setConfig] = useState<Config | null>(null)
  const [volumes, setVolumes] = useState<Volume[]>([])
  const [selected, setSelected] = useState<number | null>(null)
  const [narrow, setNarrow] = useState(() => window.innerWidth <= narrowWidthPx)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const host = `http://${window.location.hostname}:${port}`

  useEffect(() => {
    let cancelled = false

    async function load() {
      try {
        const response = await fetch(`${host}/config`)
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`)
        }
        const loaded = (await response.json()) as Config
        if (!cancelled) {
          setConfig(loaded)
          setVolumes(loaded.volumes ?? [])
        }
      } catch (err) {
        if (!cancelled) {
          setError(`${err}`)
        }
      }
    }

    load()
    return () => {
      cancelled = true
    }
  }, [host])

  useEffect(() => {
    const onResize = () => setNarrow(window.innerWidth <= narrowWidthPx)
    window.addEventListener('resize', onResize)
    return () => window.removeEventListener('resize', onResize)
  }, [])

  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    const scene = new THREE.Scene()
    scene.background = new THREE.Color(0x000000)

    const camera = new THREE.PerspectiveCamera(60, 1, 0.1, 100000)
    camera.position.set(60, 45, 60)
    cameraRef.current = camera

    const renderer = new THREE.WebGLRenderer({ antialias: true })
    renderer.setPixelRatio(window.devicePixelRatio)
    renderer.domElement.style.display = 'block'
    container.appendChild(renderer.domElement)

    const controls = new OrbitControls(camera, renderer.domElement)
    controls.enableDamping = true
    controlsRef.current = controls

    const group = new THREE.Group()
    scene.add(group)
    groupRef.current = group

    const axes = new THREE.AxesHelper(1)
    scene.add(axes)
    axesRef.current = axes

    const resize = () => {
      const { clientWidth, clientHeight } = container
      if (clientWidth === 0 || clientHeight === 0) return
      camera.aspect = clientWidth / clientHeight
      camera.updateProjectionMatrix()
      renderer.setSize(clientWidth, clientHeight, false)
    }
    resize()

    const observer = new ResizeObserver(resize)
    observer.observe(container)

    let frame = 0
    const animate = () => {
      frame = requestAnimationFrame(animate)
      controls.update()
      renderer.render(scene, camera)
    }
    animate()

    return () => {
      cancelAnimationFrame(frame)
      observer.disconnect()
      controls.dispose()
      axes.dispose()
      renderer.dispose()
      container.removeChild(renderer.domElement)
      groupRef.current = null
      cameraRef.current = null
      controlsRef.current = null
      axesRef.current = null
    }
  }, [])

  useEffect(() => {
    const group = groupRef.current
    const camera = cameraRef.current
    const controls = controlsRef.current
    const axes = axesRef.current
    if (!group || !camera || !controls || !axes) return

    // Draw enter and exit boundaries as white wireframe spheres.
    const enterMaterial = new THREE.MeshBasicMaterial({ color: 0xffffff, wireframe: true })
    const exitMaterial = new THREE.MeshBasicMaterial({ color: 0xffffff, opacity: 0.35, transparent: true, wireframe: true })
    const pickables: THREE.Mesh[] = []
    volumes.forEach((volume, index) => {
      for (const [radius, material] of [[volume.enter_radius, enterMaterial], [volume.exit_radius, exitMaterial]] as const) {
        const geometry = new THREE.SphereGeometry(Math.max(radius, 0.01), 24, 16)
        const mesh = new THREE.Mesh(geometry, material)
        mesh.position.set(volume.position.x, volume.position.y, volume.position.z)
        // Lets the raycaster map a hit back to its configuration entry.
        mesh.userData.volumeIndex = index
        group.add(mesh)
        pickables.push(mesh)
      }
    })
    pickablesRef.current = pickables

    // Scale the axes and frame the camera around the configured volumes.
    const bounds = new THREE.Box3().setFromObject(group)
    const center = bounds.isEmpty() ? new THREE.Vector3() : bounds.getCenter(new THREE.Vector3())
    const size = bounds.isEmpty() ? 0 : bounds.getSize(new THREE.Vector3()).length()
    const extent = Math.max(size, 20)

    axes.scale.setScalar(extent / 2)
    // Frame only once so editing values does not move the scene away from the pointer.
    if (!framedRef.current && volumes.length > 0) {
      framedRef.current = true
      controls.target.copy(center)
      camera.position.copy(center).add(new THREE.Vector3(1, 0.8, 1).setLength(extent))
    }
    controls.update()

    return () => {
      for (const child of [...group.children]) {
        group.remove(child)
        if (child instanceof THREE.Mesh) {
          child.geometry.dispose()
        }
      }
      pickablesRef.current = []
      enterMaterial.dispose()
      exitMaterial.dispose()
    }
  }, [volumes])

  /** Selects the sphere closest to the viewer under the pointer, or clears the selection. */
  function handlePointerUp(event: ReactPointerEvent<HTMLDivElement>) {
    const down = pointerDownRef.current
    pointerDownRef.current = null
    const container = containerRef.current
    const camera = cameraRef.current
    if (!down || !container || !camera) return
    // Ignore the pointer up that ends an orbit drag.
    if (Math.hypot(event.clientX - down.x, event.clientY - down.y) > clickSlopPx) return

    const rect = container.getBoundingClientRect()
    const pointer = new THREE.Vector2(
      ((event.clientX - rect.left) / rect.width) * 2 - 1,
      -((event.clientY - rect.top) / rect.height) * 2 + 1,
    )
    const raycaster = raycasterRef.current
    raycaster.setFromCamera(pointer, camera)
    const hit = raycaster.intersectObjects(pickablesRef.current, false)[0]

    setSelected(hit ? (hit.object.userData.volumeIndex as number) : null)
  }

  function updateValue(index: number, key: 'name' | 'enter_radius' | 'exit_radius' | 'coordinate_system', value: string | number) {
    setVolumes((current) => current.map((volume, i) => (i === index ? { ...volume, [key]: value } : volume)))
  }

  function updatePosition(index: number, axis: 'x' | 'y' | 'z', value: number) {
    setVolumes((current) => current.map((volume, i) => (
      i === index ? { ...volume, position: { ...volume.position, [axis]: value } } : volume
    )))
  }

  async function save() {
    if (!config) return
    // The backend replaces the whole config, so keep every other section untouched.
    const updated: Config = { ...config, volumes }
    setSaving(true)
    try {
      const response = await fetch(`${host}/set_config`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(updated),
      })
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`)
      }
      setConfig(updated)
      setError(null)
    } catch (err) {
      setError(`Failed to save: ${err}`)
    } finally {
      setSaving(false)
    }
  }

  const volume = selected === null ? undefined : volumes[selected]

  return (
    <div
      data-testid="view-layout"
      style={{
        display: 'flex',
        flexDirection: narrow ? 'column' : 'row',
        width: '100%',
        height: '100%',
        overflow: 'hidden',
      }}
    >
      <div
        ref={containerRef}
        data-testid="view-container"
        onPointerDown={(event) => { pointerDownRef.current = { x: event.clientX, y: event.clientY } }}
        onPointerUp={handlePointerUp}
        style={{
          position: 'relative',
          flex: narrow ? '0 0 50%' : '1 1 auto',
          minWidth: 0,
          minHeight: 0,
          background: '#000',
          color: '#fff',
          overflow: 'hidden',
        }}
      >
        {error && <p style={{ position: 'absolute', top: 8, left: 8, color: '#e66' }}>{error}</p>}
      </div>

      {selected !== null && volume && (
        <aside
          data-testid="volume-panel"
          style={{
            flex: narrow ? '0 0 50%' : '0 0 20rem',
            minWidth: 0,
            minHeight: 0,
            padding: '0.75rem',
            background: '#111',
            color: '#fff',
            borderLeft: narrow ? undefined : '1px solid #ffffff22',
            borderTop: narrow ? '1px solid #ffffff22' : undefined,
            overflowY: 'auto',
            boxSizing: 'border-box',
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <h3 data-testid="volume-panel-title" style={{ margin: 0 }}>{volume.name}</h3>
            <button
              type="button"
              data-testid="volume-panel-close"
              aria-label="Close"
              onClick={() => setSelected(null)}
              style={{ background: 'none', border: 'none', color: '#fff', cursor: 'pointer', fontSize: '1.1rem' }}
            >
              ✕
            </button>
          </div>

          <label style={rowStyle}>
            Name
            <input
              style={inputStyle}
              data-testid="volume-panel-name"
              type="text"
              value={volume.name}
              onChange={(e) => updateValue(selected, 'name', e.target.value)}
            />
          </label>

          {(['x', 'y', 'z'] as const).map((axis) => (
            <label key={axis} style={rowStyle}>
              {axis.toUpperCase()}
              <input
                style={inputStyle}
                data-testid={`volume-panel-${axis}`}
                type="number"
                step="0.1"
                value={volume.position[axis]}
                onChange={(e) => updatePosition(selected, axis, Number(e.target.value))}
              />
            </label>
          ))}

          <label style={rowStyle}>
            Enter radius
            <input
              style={inputStyle}
              data-testid="volume-panel-enter-radius"
              type="number"
              step="0.1"
              value={volume.enter_radius}
              onChange={(e) => updateValue(selected, 'enter_radius', Number(e.target.value))}
            />
          </label>

          <label style={rowStyle}>
            Exit radius
            <input
              style={inputStyle}
              data-testid="volume-panel-exit-radius"
              type="number"
              step="0.1"
              value={volume.exit_radius}
              onChange={(e) => updateValue(selected, 'exit_radius', Number(e.target.value))}
            />
          </label>

          <label style={rowStyle}>
            Coordinate system
            <input
              style={inputStyle}
              data-testid="volume-panel-coordinate-system"
              type="text"
              value={volume.coordinate_system}
              onChange={(e) => updateValue(selected, 'coordinate_system', e.target.value)}
            />
          </label>

          <div style={{ ...rowStyle, justifyContent: 'flex-end' }}>
            <button type="button" data-testid="volume-panel-save" disabled={saving} onClick={save}>
              {saving ? 'Saving...' : 'Save'}
            </button>
          </div>
        </aside>
      )}
    </div>
  )
}
