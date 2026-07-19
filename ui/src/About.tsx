
import { useEffect, useState } from "react";
import type { BuildInfo } from "./types/BuildInfo"

export interface AboutProps {
  port: number
}

export function About({ port }: AboutProps) {

  const protocol = 'http'
  const rootUrl = window.location.hostname
  const rootPort = port
  const host = `${protocol}://${rootUrl}:${rootPort}`
  const [buildInfo, setBuildInfo] = useState<BuildInfo | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<any>(null);

  useEffect(() => {
    async function loadData() {
      try {
        const response = await fetch(`${host}/build_info.json`);

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }

        const buildInfo = await response.json();
        setBuildInfo(buildInfo);
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

  function version() {
    return buildInfo ? `${buildInfo.cargo_pkg_version}-${buildInfo.git_hash}` : ''
  }

  function verSpan() {
    return (<span style={{color: '#999'}}>
      {version()}
    </span>)
  }

  return (
    <div>
      <h1>About {verSpan()}</h1>
      {buildInfo ? Object.entries(buildInfo).map((info, index) => {
        return <div key={index}>
          <strong>{info[0]}:</strong> {(info[1] as any).toString()}
        </div>
      }) : <p>Loading build information...</p>}
    </div>
  );
}
