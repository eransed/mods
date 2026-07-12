import { useEffect, useState } from 'react'
import { NavLink, Navigate, Route, Routes } from 'react-router-dom'
import Overview from './Overview'
import { About } from './About'

type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error'

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
  {
    path: '/about',
    label: 'About',
    description: 'Application information.',
  },
]

function App() {
  const [status, setStatus] = useState<ConnectionState>('connecting')
  const [reconnectAttempts, setReconnectAttempts] = useState(0)
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
      let gotConfig = false
      try {
        console.log('Fetching config...')
        const response = await fetch(`${host}/config`)
        if (response.ok) {
          const config = (await response.json()) as ConfigResponse
          if (typeof config.ws_port === 'number') {
            console.log('Config received:', config)

            resolvedWsPort = config.ws_port
            gotConfig = true
            setReconnectAttempts(0)
          } else {
            console.error('ws_port is not a number in config response')
          }
        } else {
          console.error('Failed to fetch config (non ok response):', response.statusText)
        }
      } catch (e) {
        // Keep the default port when /config is unavailable.
        console.error('Failed to fetch config (exception):', e)
      }

      if (isCancelled) {
        console.log('Connection attempt cancelled, not connecting to WebSocket.')
        return
      }

      let reconnectTimeout: any = null

      function handleReconnect() {
        if (reconnectTimeout) {
          console.log('Clearing existing reconnect timeout')
          clearTimeout(reconnectTimeout)
        }

        if (status === 'connected') {
          console.log('Already connected, no need to reconnect.')
          return
        }

        const reconnectDelayMs = 3000

        console.log(`Scheduling reconnect in ${reconnectDelayMs} seconds`)
        reconnectTimeout = setTimeout(() => {
          setReconnectAttempts((prev) => prev + 1)
          connect()
        }, reconnectDelayMs)
      }

      if (gotConfig) {
        console.log('Connecting to WebSocket on port', resolvedWsPort)
        setWsPort(resolvedWsPort)
        const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws'
        const hostname = window.location.hostname || '127.0.0.1'
        const wsUrl =
          import.meta.env.VITE_WS_URL ?? `${protocol}://${hostname}:${resolvedWsPort}`

        socket = new WebSocket(wsUrl)

        socket.onopen = () => {
          setStatus('connected')
        }

        socket.onclose = () => {
          setStatus('disconnected')
          handleReconnect()
        }

        socket.onerror = () => {
          setStatus('error')
          handleReconnect()
        }
      } else {
        handleReconnect()
      }

    }

    connect()

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
          <h1>Oak - Event Router</h1>
          <p className="status" aria-live="polite">
            <span className={`dot dot-${status}`} aria-hidden="true" />
            {status}{reconnectAttempts > 0 ? `[${reconnectAttempts}]` : null} - {wsPort}
          </p>
        </header>

        <section className="page-body">
          <Routes>
            <Route path="/" element={<Navigate to="/overview" replace />} />
            <Route path="/overview" element={<Overview />} />
            <Route path="/about" element={<About />} />
            <Route path="/about" element={<About />} />
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
