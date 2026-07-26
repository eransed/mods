
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
    return <p>Error</p>;
  }

  return (
    <div>
      <h1>API</h1>
      {endpoints ? Object.entries(endpoints).map((endpoint, index) => {
        return <div key={index}>
        <a style={{ color: '#fff' }} href={`${protocol}://${rootUrl}:${rootPort}${endpoint[1]}`} target="_blank" rel="noopener noreferrer">
          {endpoint[1]}
        </a>
        </div>
      }) : <p>Loading endpoints...</p>}
    </div>
  );
}
