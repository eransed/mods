import { useEffect, useState } from 'react'
import { NavLink, Navigate, Route, Routes } from 'react-router-dom'

type ConnectionState = 'connecting' | 'connected' | 'disconnected'

type ConfigResponse = {
  ws_port?: number
}

const pages = [
  {
    path: '/overview',
    label: 'Overview',
    description: '3D overview of the station.',
  },
  {
    path: '/volumes',
    label: 'Volumes',
    description: 'Manage volumes (positions). List view.',
  },
  {
    path: '/geometries',
    label: 'Geometries',
    description: 'Tool geometries, including articulated arms.',
  },
  {
    path: '/device-connections',
    label: 'Device Connections',
    description: 'Tightening, welding tools, projectors etc.',
  },
  {
    path: '/sensors',
    label: 'Sensors',
    description: 'Configure different types of sensors.',
  },
  {
    path: '/message-router',
    label: 'Message Router',
    description: 'Manage message routing and preferences.',
  },
  {
    path: '/message-log',
    label: 'Message Log',
    description: 'View and manage message logs.',
  },
  {
    path: '/settings',
    label: 'Settings',
    description: 'Manage runtime behavior and preferences.',
  },
]

function App() {
  const [status, setStatus] = useState<ConnectionState>('connecting')
  const [wsPort, setWsPort] = useState(8085)

  const protocol = 'http'
  const rootUrl = 'localhost'
  const rootPort = 8080
  const host = `${protocol}://${rootUrl}:${rootPort}`

  const defaultWsPort = 8085

  useEffect(() => {
    let socket: WebSocket | null = null
    let isCancelled = false

    const connect = async () => {
      let resolvedWsPort = defaultWsPort

      try {
        const response = await fetch(`${host}/config`)
        if (response.ok) {
          const config = (await response.json()) as ConfigResponse
          if (typeof config.ws_port === 'number') {
            resolvedWsPort = config.ws_port
          } else {
            console.warn('ws_port is not a number in config response')
          }
        } else {
          console.error('Failed to fetch config:', response.statusText)
        }
      } catch (e) {
        // Keep the default port when /config is unavailable.
        console.error('Failed to fetch config:', e)
      }

      if (isCancelled) {
        console.log('Connection attempt cancelled, not connecting to WebSocket.')
        return
      }

      setWsPort(resolvedWsPort)

      const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws'
      const hostname = window.location.hostname || '127.0.0.1'
      const wsUrl =
        import.meta.env.VITE_WS_URL ?? `${protocol}://${hostname}:${resolvedWsPort}`

      socket = new WebSocket(wsUrl)

      socket.onopen = () => setStatus('connected')
      socket.onclose = () => setStatus('disconnected')
      socket.onerror = () => setStatus('disconnected')
    }

    void connect()

    return () => {
      isCancelled = true
      socket?.close()
    }
  }, [defaultWsPort, host])

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <h2>Menu</h2>
        <nav aria-label="Primary">
          <ul className="nav-list">
            {pages.map((page) => (
              <li key={page.path}>
                <NavLink
                  to={page.path}
                  className={({ isActive }) =>
                    `nav-link${isActive ? ' nav-link-active' : ''}`
                  }
                >
                  {page.label}
                </NavLink>
              </li>
            ))}
          </ul>
        </nav>
      </aside>

      <main className="content">
        <header className="header">
          <h1>EventRouter</h1>
          <p className="status" aria-live="polite">
            <span className={`dot dot-${status}`} aria-hidden="true" />
            {status} - {wsPort}
          </p>
        </header>

        <section className="page-body">
          <Routes>
            <Route path="/" element={<Navigate to="/overview" replace />} />
            {pages.map((page) => (
              <Route
                key={page.path}
                path={page.path}
                element={
                  <article>
                    <h2>{page.label}</h2>
                    <p>{page.description}</p>
                  </article>
                }
              />
            ))}
          </Routes>
        </section>
      </main>
    </div>
  )
}

export default App
