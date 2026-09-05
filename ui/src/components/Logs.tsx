import { useEffect, useRef, useState } from 'react'

interface LiveLog {
  topic?: string
  timestamp?: string
  level?: string
  message?: string
}

interface LogsResponse {
  logs: string[]
  page: number
  page_size: number
  total_logs: number
  has_previous: boolean
  has_next: boolean
}

interface LogsProps {
  port: number
  webSocket: WebSocket | null
}

const MAX_LIVE_LOGS = 10_000

export function Logs({ port, webSocket }: LogsProps) {
  const [page, setPage] = useState(1)
  const [history, setHistory] = useState<LogsResponse | null>(null)
  const [liveLogs, setLiveLogs] = useState<string[]>([])
  const [liveViewEnabled, setLiveViewEnabled] = useState(true)
  const [newestAtTop, setNewestAtTop] = useState(true)
  const [reloadKey, setReloadKey] = useState(0)
  const [error, setError] = useState<string | null>(null)
  const logsListRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    let cancelled = false

    async function loadLogs() {
      try {
        const response = await fetch(`http://${window.location.hostname}:${port}/api/logs?page=${page}`)
        if (!response.ok) throw new Error(`HTTP ${response.status}`)
        const entries = await response.json() as LogsResponse
        if (!cancelled) setHistory(entries)
      } catch (loadError) {
        if (!cancelled) setError(String(loadError))
      }
    }

    void loadLogs()
    return () => { cancelled = true }
  }, [page, port, reloadKey])

  useEffect(() => {
    if (!webSocket || !liveViewEnabled) return

    const handleMessage = (event: MessageEvent<string>) => {
      try {
        const log = JSON.parse(event.data) as LiveLog
        if (log.topic !== 'log' || !log.message) return
        const timestamp = log.timestamp ? `${log.timestamp} ` : ''
        const level = log.level ? `${log.level.toUpperCase()} ` : ''
        setLiveLogs((current) => [`${timestamp}${level}${log.message}`, ...current].slice(0, MAX_LIVE_LOGS))
      } catch { }
    }

    webSocket.addEventListener('message', handleMessage)
    return () => webSocket.removeEventListener('message', handleMessage)
  }, [liveViewEnabled, webSocket])

  const pageLogs = page === 1
    ? [...liveLogs, ...(history?.logs ?? [])].slice(0, history?.page_size ?? liveLogs.length)
    : (history?.logs ?? [])
  const orderedLogs = [...pageLogs]
  if (!newestAtTop) orderedLogs.reverse()
  const visibleLogs = orderedLogs.map((log, index) => ({
    log,
    lineNumber: (page - 1) * (history?.page_size ?? 0) + index + 1,
  }))
  const lastPage = history ? Math.max(1, Math.ceil(history.total_logs / history.page_size)) : page

  function setLiveView(enabled: boolean) {
    setLiveViewEnabled(enabled)
    if (enabled) {
      setLiveLogs([])
      setPage(1)
      setReloadKey((current) => current + 1)
    }
  }

  function scrollLogs(position: 'top' | 'bottom') {
    const logsList = logsListRef.current
    if (!logsList) return
    logsList.scrollTo({
      top: position === 'top' ? 0 : logsList.scrollHeight,
      behavior: 'smooth',
    })
  }

  return (
    <section className="logs-page">
      <div className="logs-heading">
        <div>
          <h2>Logs</h2>
          <p className="logs-status" aria-live="polite">
            {!liveViewEnabled ? 'Paused' : webSocket ? 'Live' : 'Waiting for websocket connection'}
            {error ? ` - ${error}` : null}
          </p>
        </div>
        <div className="logs-controls">
          <button type="button" onClick={() => scrollLogs('top')}>Top</button>
          <button type="button" onClick={() => scrollLogs('bottom')}>Bottom</button>
          <label className="logs-toggle">
            <span>Live view</span>
            <input
              type="checkbox"
              role="switch"
              checked={liveViewEnabled}
              onChange={(event) => setLiveView(event.target.checked)}
            />
          </label>
          <label className="logs-toggle">
            <span>Newest at top</span>
            <input
              type="checkbox"
              role="switch"
              checked={newestAtTop}
              onChange={(event) => setNewestAtTop(event.target.checked)}
            />
          </label>
        </div>
        <span className="logs-count">{visibleLogs.length} visible / {history?.total_logs ?? 0} entries</span>
      </div>
      <div ref={logsListRef} className="logs-list" aria-live="polite">
        {visibleLogs.length === 0 && !error ? <p className="logs-empty">No log entries</p> : null}
        {visibleLogs.map(({ log, lineNumber }, index) => (
          <pre key={`${log}-${index}`}><span className="logs-line-number">{lineNumber}</span><span>{log}</span></pre>
        ))}
      </div>
      <div className="logs-pagination">
        <button type="button" onClick={() => setPage(1)} disabled={!history?.has_previous}>
          First
        </button>
        <button type="button" onClick={() => setPage((current) => current - 1)} disabled={!history?.has_previous}>
          Previous
        </button>
        <span>Page {history?.page ?? page} / {lastPage}</span>
        <button type="button" onClick={() => setPage((current) => current + 1)} disabled={!history?.has_next}>
          Next
        </button>
        <button type="button" onClick={() => setPage(lastPage)} disabled={!history?.has_next}>
          Last
        </button>
      </div>
    </section>
  )
}
