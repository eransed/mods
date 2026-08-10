
import { useEffect, useState } from "react";

export interface ApiProps {
  port: number
}

export function Api({ port }: ApiProps) {

  const protocol = 'http'
  const rootUrl = window.location.hostname
  const rootPort = port
  const host = `${protocol}://${rootUrl}:${rootPort}`
  const [endpoints, setEndpoints] = useState<Record<string, any> | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<any>(null);
  const [log, setLog] = useState<string[] | null>(null);

  function get(url: string) {
    fetch(url)
      .then(data => data.text())
      .then(data => {
        let e = data
        console.log(e)
        try {
          let j = JSON.parse(e)
          e = JSON.stringify(j, null, 2)
          console.log(e)
        } catch { }
        let d = new Date()
        let ts = `${d.toLocaleTimeString('sv-SE')}.${d.getMilliseconds().toFixed(0).padStart(3, '0')}`
        let entry = `${ts} GET ${url} => ${e}`
        setLog(prevLog => [entry, ...(prevLog || [])]);
      })
      .catch((error) => {
        let m = `Error fetching API endpoint: ${error}`
        console.error(m)
        setLog(prevLog => [m, ...(prevLog || [])]);
      });
  }


  useEffect(() => {
    async function loadData() {
      try {
        const response = await fetch(`${host}/endpoints`);

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }

        const endpoints = await response.json();
        setEndpoints(endpoints);
      } catch (err) {
        setError(err);
      } finally {
        setLoading(false);
      }
    }

    loadData();
  }, []); // Run once when the component mounts

  if (loading) {
    return <p>Loading...</p>;
  }

  if (error) {
    return <>
      <div>
        <h1>API</h1>
        <p>{`${error}`}</p>
      </div>
    </>
  }

  return (
    <div style={{ display: 'grid', gridTemplateColumns: '1fr 3fr', gap: '10px' }}>
      <div>
        <h1>API</h1>
        {endpoints ? Object.entries(endpoints).map((endpoint, index) => {
          return <div key={index}>
            <button style={{ margin: '8px', backgroundColor: '#007bff', color: '#fff', border: 'none', borderRadius: '4px', padding: '5px', cursor: 'pointer' }} onClick={() => get(`${host}${endpoint[1]}`)}>
              GET
            </button>
            <a style={{ fontSize: '0.8rem', color: '#fff' }} href={`${protocol}://${rootUrl}:${rootPort}${endpoint[1]}`} target="_blank" rel="noopener noreferrer">
              {endpoint[1]}
            </a>

          </div>
        }) : <p>Loading api endpoints...</p>}
      </div>

      <div style={{ overflowY: 'scroll', minHeight: '600px', maxHeight: '600px' }}>
        <div style={{ display: 'flex' }}>
          <div style={{ fontSize: '1.1rem' }}>Log</div>
          <button style={{ marginLeft: '20px' }} onClick={() => {
            setLog([])
          }}>Clear</button>
        </div>
        {log ? log.map((entry, index) => <pre style={{ fontSize: '0.7rem' }} key={index}>{entry}</pre>) : null}
      </div>
    </div>
  );
}

