import { useEffect, useRef, useState } from 'react'
import { NavLink, Navigate, Route, Routes } from 'react-router-dom'
import Overview from './Overview'
import { About } from './About'
import { msPretty } from './lib/utils'
import { Api } from './Api'
import { Camera } from './Camera'

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
  {
    path: '/api',
    label: 'API',
    description: 'View available API endpoints.',
  },
  {
    path: '/camera',
    label: 'Camera',
    description: 'View camera feed.',
  }
]

function App() {
  console.log('App')
  const [status, setStatus] = useState<ConnectionState>('connecting')
  const [reconnectAttempts, setReconnectAttempts] = useState(0)
  const [websocket, setWebsocket] = useState<WebSocket | null>(null)
  const disconnectStart = useRef<number | null>(null)
  const websocketRef = useRef<WebSocket | null>(null)
  const [disconnectedSince, setDisconnectedSince] = useState(0)
  const [wsPort, setWsPort] = useState(8124)

  const protocol = 'http'
  const rootUrl = window.location.hostname

  let rootPort = parseInt(window.location.port || '8123', 10)
  // the root http port when running the dev server must be set to the default port of 8123
  // determine if we are running in dev mode or production mode based on the port number
  if (import.meta.env) {
    rootPort = 8123
  }

  const host = `${protocol}://${rootUrl}:${rootPort}`

  const defaultWsPort = 8124

  useEffect(() => {
    console.log('useEffect')
    let isCancelled = false

    const connect = async () => {
      console.log('Connecting...')
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
            disconnectStart.current = null
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

        if (disconnectStart.current === null) {
          disconnectStart.current = performance.now()
          console.log("Setting the disconnected time:", disconnectStart)
        }

        setDisconnectedSince(
          performance.now() - disconnectStart.current
        )

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

        const newWebSocket = new WebSocket(wsUrl)

        newWebSocket.onopen = () => {
          console.log('websocket open')
          setStatus('connected')
          disconnectStart.current = null
          setReconnectAttempts(0)
        }

        newWebSocket.onclose = () => {
          console.warn('websocket closed')
          setStatus('disconnected')
          setWebsocket(null)
          handleReconnect()
        }

        newWebSocket.onerror = () => {
          console.error('websocket error')
          setStatus('error')
          handleReconnect()
        }

        // newWebSocket.onmessage = (event) => {
        //   console.log('websocket message received:', event.data)
        // }

        websocketRef.current = newWebSocket
        setWebsocket(newWebSocket)
      } else {
        console.warn("Could not fetch the config, will not connect to websocket")
        handleReconnect()
      }

    }

    console.log('Prime connect')
    connect()

    return () => {
      console.log('Cancelled')
      isCancelled = true
      websocketRef.current?.close()
    }
  }, [defaultWsPort, host])

  console.log('Render')
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
            {status}{reconnectAttempts > 0 ? `[${msPretty(disconnectedSince)}]` : null} - {wsPort}
          </p>
        </header>

        <section className="page-body">
          <Routes>
            <Route path="/" element={<Navigate to="/overview" replace />} />
            <Route path="/overview" element={<Overview />} />
            <Route path="/camera" element={websocket ? <Camera webSocket={websocket} /> : null} />
            <Route path="/about" element={<About port={rootPort} />} />
            <Route path="/api" element={<Api port={rootPort} />} />
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
