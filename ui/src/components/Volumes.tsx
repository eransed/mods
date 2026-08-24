import { useEffect, useState, type CSSProperties } from 'react'
import type { Config, Volume } from '../types/Config'
import { Button } from './Button'

export interface VolumesProps {
  port: number
}

const cellStyle: CSSProperties = {
  padding: '0.35rem 0.6rem',
  borderBottom: '1px solid #ffffff22',
  textAlign: 'left',
  verticalAlign: 'middle',
}

const numberInputStyle: CSSProperties = { width: '6rem' }

function newVolume(existing: Volume[]): Volume {
  const used = new Set(existing.map((volume) => volume.name))
  let index = 1
  while (used.has(`sphere_${index}`)) index += 1
  return {
    name: `sphere_${index}`,
    position: { x: Math.random() * 200, y: Math.random() * 200, z: Math.random() * 200 },
    enter_radius: Math.random() * 4 + 1,
    exit_radius: Math.random() * 8 + 2,
    coordinate_system: 'world',
  }
}

/** Specialized editor for the compact sphere volumes stored in the backend config. */
export function Volumes({ port }: VolumesProps) {
  const host = `http://${window.location.hostname}:${port}`
  const [config, setConfig] = useState<Config | null>(null)
  const [volumes, setVolumes] = useState<Volume[]>([])
  const [error, setError] = useState<string | null>(null)
  const [saving, setSaving] = useState(false)

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

  const modified = config !== null && JSON.stringify(config.volumes ?? []) !== JSON.stringify(volumes)

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

  if (!config && !error) {
    return (
      <div>
        <h2>Volumes</h2>
        <p>Loading...</p>
      </div>
    )
  }

  return (
    <div>
      <h2>Volumes{modified ? '*' : ''}</h2>
      <p>Manage sphere volumes. List view.</p>
      {error && <p style={{ color: '#e66' }}>{error}</p>}

      <div style={{ display: 'flex', gap: '0.5rem', margin: '0.5rem 0' }}>
        <Button type="button" onClick={() => setVolumes((current) => [...current, newVolume(current)])}>
          Add Sphere
        </Button>
        <Button type="button" disabled={!modified} onClick={() => setVolumes(config?.volumes ?? [])}>
          Undo
        </Button>
        <Button variant="primary" type="button" disabled={!modified || saving} onClick={save}>
          {saving ? 'Saving...' : 'Save'}
        </Button>
      </div>

      {volumes.length === 0 ? (
        <p>No volumes configured.</p>
      ) : (
        <table style={{ borderCollapse: 'collapse', width: '100%' }}>
          <thead>
            <tr>
              <th style={cellStyle}>Name</th>
              <th style={cellStyle}>X</th>
              <th style={cellStyle}>Y</th>
              <th style={cellStyle}>Z</th>
              <th style={cellStyle}>Enter radius</th>
              <th style={cellStyle}>Exit radius</th>
              <th style={cellStyle}>Coordinate system</th>
              <th style={cellStyle}></th>
            </tr>
          </thead>
          <tbody>
            {volumes.map((volume, index) => (
              <tr key={index}>
                <td style={cellStyle}>
                  <input
                    type="text"
                    value={volume.name}
                    onChange={(e) => updateValue(index, 'name', e.target.value)}
                  />
                </td>
                {(['x', 'y', 'z'] as const).map((axis) => (
                  <td key={axis} style={cellStyle}>
                    <input
                      style={numberInputStyle}
                      type="number"
                      step="0.1"
                      value={volume.position[axis]}
                      onChange={(e) => updatePosition(index, axis, Number(e.target.value))}
                    />
                  </td>
                ))}
                <td style={cellStyle}>
                  <input
                    style={numberInputStyle}
                    type="number"
                    step="0.1"
                    value={volume.enter_radius}
                    onChange={(e) => updateValue(index, 'enter_radius', Number(e.target.value))}
                  />
                </td>
                <td style={cellStyle}>
                  <input
                    style={numberInputStyle}
                    type="number"
                    step="0.1"
                    value={volume.exit_radius}
                    onChange={(e) => updateValue(index, 'exit_radius', Number(e.target.value))}
                  />
                </td>
                <td style={cellStyle}>
                  <input
                    type="text"
                    value={volume.coordinate_system}
                    onChange={(e) => updateValue(index, 'coordinate_system', e.target.value)}
                  />
                </td>
                <td style={cellStyle}>
                  <Button
                    type="button"
                    onClick={() => setVolumes((current) => current.filter((_, i) => i !== index))}
                  >
                    Remove
                  </Button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  )
}
