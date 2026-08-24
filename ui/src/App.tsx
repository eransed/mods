import { useEffect, useRef, useState } from 'react'
import { NavLink, Navigate, Route, Routes } from 'react-router-dom'
import { msPretty } from './lib/utils'
import { Overview } from './components/Overview'
import { Api } from './components/Api'
import { Camera } from './components/Camera'
import { View } from './components/View'
import { Volumes } from './components/Volumes'
import { Settings, type OpenProtocolState } from './components/Settings'
import { About } from './components/About'
import { Button } from './components/Button'
import type { Config } from './types/Config'
// import mermaid from 'mermaid'

type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'error'

type SystemStats = {
  cpu: string
  ram: string
  mem: string
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
  },
  {
    path: '/view',
    label: 'View',
    description: 'Full page 3D view.',
  },
  {
    path: '/docs',
    label: 'Docs',
    description: 'Documentation',
  }
]

function Docs() {
  // setTimeout(() => {
  //   mermaid.run();
  // }, 300)

  let m =
    `
  graph TD
    A[Client] --> B[Load Balancer]
    B --> C[Server01]
    B --> D[Server02]

  `
  return <>
    <div>
      <pre className="mermaid">
        {m}
      </pre>
    </div>
  </>
}

function App() {
  // mermaid.initialize({})
  const cutoffWidth = 600

  const [status, setStatus] = useState<ConnectionState>('connecting')
  const [reconnectAttempts, setReconnectAttempts] = useState(0)
  const [systemStats, setSystemStats] = useState<SystemStats>({ cpu: '-', ram: '-', mem: '-' })
  const [websocket, setWebsocket] = useState<WebSocket | null>(null)
  const [openProtocolStates, setOpenProtocolStates] = useState<Record<string, OpenProtocolState>>({})
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
  if (!import.meta.env.PROD) {
    rootPort = 8123
  }

  const host = `${protocol}://${rootUrl}:${rootPort}`

  const defaultWsPort = 8124

  useEffect(() => {
    const handleResize = () => {
      console.log('RESIZE')
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
    if (import.meta.env.PROD) {
      console.log('Running in production mode, using port from window.location.port =', window.location.port)
    } else {
      console.log('Running in dev mode, using default port 8123 for http')
      console.log('import.meta.env:', import.meta.env)
    }
    console.log('host:', host)

    let isCancelled = false

    const connect = async () => {
      console.log('Connecting...')
      let resolvedWsPort = defaultWsPort
      let gotConfig = false
      try {
        console.log('Fetching config...')
        const response = await fetch(`${host}/config`)
        if (response.ok) {
          // Read the websocket port from the generated ConfigProperty shape.
          const config = (await response.json()) as Config
          if (typeof config.general_config?.ws_port?.value === 'number') {
            console.log('Config received:', config)
            resolvedWsPort = config.general_config.ws_port.value
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

        newWebSocket.onmessage = (e) => {
          try {
            let o = JSON.parse(e.data)
            if (o.OpenProtocolState) {
              const state = o.OpenProtocolState as OpenProtocolState
              setOpenProtocolStates((current) => ({ ...current, [state.name]: state }))
            }
            if (o.SystemStatus) {
              let cpu = (o.SystemStatus.cpu_percent as number).toFixed(1)
              let ram = (o.SystemStatus.ram_percent as number).toFixed(1)
              let mem = (o.SystemStatus.pid_mem_bytes as number / 1024 / 1024).toFixed(1)
              setSystemStats({ cpu, ram, mem })
            }
          } catch { }
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
          <Button
            className="hamburger-btn"
            onClick={toggleSidebar}
            aria-label="Toggle menu"
            aria-expanded={sidebarOpen}
          >
            <span></span>
            <span></span>
            <span></span>
          </Button>
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
            <Button
              className="hamburger-btn header-hamburger"
              onClick={toggleSidebar}
              aria-label="Toggle menu"
              aria-expanded={sidebarOpen}
            >
              <span></span>
              <span></span>
              <span></span>
            </Button>
          )}
          <div className="header-title">
            <h1>mods</h1>
            <p className="status" aria-live="polite">
              <span className={`dot dot-${status}`} aria-hidden="true" />
              <span className="status-stats">
                <span className="status-stat">CPU: {systemStats.cpu}%</span>
                <span className="status-stat">RAM: {systemStats.ram}%</span>
                <span className="status-stat">MEM: {systemStats.mem}MB</span>
              </span>
              <span className="status-text">
                {status}{reconnectAttempts > 0 ? `[${msPretty(disconnectedSince)}]` : null} - {wsPort}
              </span>
            </p>
          </div>
        </header>

        <section className="page-body">
          <Routes>
            <Route path="/" element={<Navigate to="/overview" replace />} />
            <Route path="/overview" element={<Overview />} />
            <Route path="/docs" element={<Docs />} />
            <Route path="/settings" element={<Settings http_port={rootPort} webSocket={websocket} openProtocolStates={openProtocolStates} />} />
            <Route path="/camera" element={websocket ? <Camera webSocket={websocket} /> : <div>Camera waiting for websocket connection...</div>} />
            <Route path="/view" element={<View port={rootPort} />} />
            <Route path="/volumes" element={<Volumes port={rootPort} />} />
            <Route path="/api" element={<Api port={rootPort} />} />
            <Route path="/about" element={<About port={rootPort} />} />
            {pages
              .filter((page) => page.path !== '/volumes')
              .map((page) => (
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
