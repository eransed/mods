import { useEffect, useRef, useState } from 'react'
import { NavLink, Navigate, Route, Routes } from 'react-router-dom'
import { msPretty } from './lib/utils'
import { Overview } from './components/Overview'
import { Api } from './components/Api'
import { Camera } from './components/Camera'
import { Settings } from './components/Settings'
import { About } from './components/About'

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
  const cutoffWidth = 600

  const [status, setStatus] = useState<ConnectionState>('connecting')
  const [reconnectAttempts, setReconnectAttempts] = useState(0)
  const [websocket, setWebsocket] = useState<WebSocket | null>(null)
  const [sidebarOpen, setSidebarOpen] = useState(window.innerWidth > cutoffWidth)
  const [screenWidth, setScreenWidth] = useState(window.innerWidth)
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
    console.log('import.meta.env:', import.meta.env)
    console.log('Running in dev mode, using default port 8123 for http')
    rootPort = 8123
  } else {
    console.log('Running in production mode, using port from window.location.port =', window.location.port)
  }

  const host = `${protocol}://${rootUrl}:${rootPort}`
  console.log('host:', host)

  const defaultWsPort = 8124

  useEffect(() => {
    const handleResize = () => {
      const newWidth = window.innerWidth
      setScreenWidth(newWidth)
      // Only auto-close menu on small screens when resizing to small
      if (newWidth <= cutoffWidth) {
        setSidebarOpen(false)
      } else {
        setSidebarOpen(true)
      }
    }

    window.addEventListener('resize', handleResize)
    return () => window.removeEventListener('resize', handleResize)
  }, [])

  const toggleSidebar = () => {
    setSidebarOpen(!sidebarOpen)
  }

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
        setWsPort(resolvedWsPort)
        const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws'
        const hostname = window.location.hostname || '127.0.0.1'
        const wsUrl = `${protocol}://${hostname}:${resolvedWsPort}`
        
        console.log('Connecting to WebSocket: ', wsUrl)
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

        websocketRef.current = newWebSocket
        setWebsocket(newWebSocket)
      } else {
        console.warn("Could not fetch the config, will not connect to websocket")
        handleReconnect()
      }

    }

    connect()

    return () => {
      console.log('Cancelled')
      isCancelled = true
      websocketRef.current?.close()
    }
  }, [defaultWsPort, host])

  return (
    <div className={`app-shell${screenWidth <= cutoffWidth ? ' small' : ''}${sidebarOpen ? ' sidebar-open' : ''}`}>
      <aside className={`sidebar${screenWidth <= cutoffWidth ? ' small' : ''}${screenWidth <= cutoffWidth && sidebarOpen ? ' visible' : ''}`}>
        <div className="sidebar-header">
          <button
            className="hamburger-btn"
            onClick={toggleSidebar}
            aria-label="Toggle menu"
            aria-expanded={sidebarOpen}
          >
            <span></span>
            <span></span>
            <span></span>
          </button>
        </div>
        <nav aria-label="Primary">
          <ul className="nav-list">
            {pages.map((page) => (
              <li key={page.path}>
                <NavLink
                  to={page.path}
                  className={({ isActive }) =>
                    `nav-link${isActive ? ' nav-link-active' : ''}`
                  }
                  onClick={() => {
                    if (screenWidth <= cutoffWidth) {
                      setSidebarOpen(false)
                    }
                  }}
                >
                  {page.label}
                </NavLink>
              </li>
            ))}
          </ul>
        </nav>
      </aside>

      <main className={`content${screenWidth <= cutoffWidth ? ' small' : ''}`}>
        <header className="header">
          {!sidebarOpen && (
            <button
              className="hamburger-btn header-hamburger"
              onClick={toggleSidebar}
              aria-label="Toggle menu"
              aria-expanded={sidebarOpen}
            >
              <span></span>
              <span></span>
              <span></span>
            </button>
          )}
          <h1>mods</h1>
          <p className="status" aria-live="polite">
            <span className={`dot dot-${status}`} aria-hidden="true" />
            {status}{reconnectAttempts > 0 ? `[${msPretty(disconnectedSince)}]` : null} - {wsPort}
          </p>
        </header>

        <section className="page-body">
          <Routes>
            <Route path="/" element={<Navigate to="/overview" replace />} />
            <Route path="/overview" element={<Overview />} />
            <Route path="/settings" element={<Settings http_port={rootPort} />} />
            <Route path="/camera" element={websocket ? <Camera webSocket={websocket} /> : null} />
            <Route path="/api" element={<Api port={rootPort} />} />
            <Route path="/about" element={<About port={rootPort} />} />
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
