import { useEffect, useState } from 'react'

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

export function Logs({ port, webSocket }: LogsProps) {
  const [page, setPage] = useState(1)
  const [history, setHistory] = useState<LogsResponse | null>(null)
  const [liveLogs, setLiveLogs] = useState<string[]>([])
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    let cancelled = false

    async function loadLogs() {
      try {
        const response = await fetch(`http://${window.location.hostname}:${port}/logs?page=${page}`)
        if (!response.ok) throw new Error(`HTTP ${response.status}`)
        const entries = await response.json() as LogsResponse
        if (!cancelled) setHistory(entries)
      } catch (loadError) {
        if (!cancelled) setError(String(loadError))
      }
    }

    void loadLogs()
    return () => { cancelled = true }
  }, [page, port])

  useEffect(() => {
    if (!webSocket) return

    const handleMessage = (event: MessageEvent<string>) => {
      try {
        const log = JSON.parse(event.data) as LiveLog
        if (log.topic !== 'log' || !log.message) return
        const timestamp = log.timestamp ? `${log.timestamp} ` : ''
        const level = log.level ? `${log.level.toUpperCase()} ` : ''
        setLiveLogs((current) => [`${timestamp}${level}${log.message}`, ...current].slice(0, 500))
      } catch { }
    }

    webSocket.addEventListener('message', handleMessage)
    return () => webSocket.removeEventListener('message', handleMessage)
  }, [webSocket])

  return (
    <section className="logs-page">
      <div className="logs-heading">
        <div>
          <h2>Logs</h2>
          <p className="logs-status" aria-live="polite">
            {webSocket ? 'Live' : 'Waiting for websocket connection'}
            {error ? ` - ${error}` : null}
          </p>
        </div>
        <span className="logs-count">{history?.total_logs ?? 0} entries</span>
      </div>
      <div className="logs-list" aria-live="polite">
        {liveLogs.length === 0 && (history?.logs.length ?? 0) === 0 && !error ? <p className="logs-empty">No log entries</p> : null}
        {(page === 1 ? [...liveLogs, ...(history?.logs ?? [])].slice(0, 500) : (history?.logs ?? [])).map((log, index) => <pre key={`${log}-${index}`}>{log}</pre>)}
      </div>
      <div className="logs-pagination">
        <button type="button" onClick={() => setPage((current) => current - 1)} disabled={!history?.has_previous}>
          Previous
        </button>
        <span>Page {history?.page ?? page}</span>
        <button type="button" onClick={() => setPage((current) => current + 1)} disabled={!history?.has_next}>
          Next
        </button>
      </div>
    </section>
  )
}
