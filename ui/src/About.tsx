// import { useEffect } from "react"
// import type { BuildInfo } from "./types/BuildInfo"

// export function About() {
//   const protocol = 'http'
//   const rootUrl = 'localhost'
//   const rootPort = 8080
//   const host = `${protocol}://${rootUrl}:${rootPort}`
//   let buildInfo : BuildInfo | null = null

//   useEffect(() => {
//     let isCancelled = false
//     const fetchBuildInfo = async () => {
//       try {
//         console.log('Fetching config...')
//         const response = await fetch(`${host}/build_info.json`)
//         if (response.ok) {
//           buildInfo = (await response.json()) as BuildInfo
//           console.log('Build info received:', buildInfo)
//         } else {
//           console.error('Failed to fetch buildInfo (non ok response):', response.statusText)
//         }
//       } catch (e) {
//         console.error('Failed to fetch buildInfo (exception):', e)
//       }
//     }

//     fetchBuildInfo()

//     if (isCancelled) {
//       return
//     }

//     return () => {
//       isCancelled = true
//     }
//   }, [buildInfo, host])

//   return <>
//     <h1>About</h1>
//     {buildInfo ? Object.entries(buildInfo).map((info, index) => {
//       return <div key={index}>
//         <strong>{info[0]}:</strong> {(info[1] as any).toString()}
//       </div>
//     }) : <p>Loading build information...</p>}

//   </>
// }


import { useEffect, useState } from "react";
import type { BuildInfo } from "./types/BuildInfo"


export function About() {

  const protocol = 'http'
  const rootUrl = 'localhost'
  const rootPort = 8080
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
    return <p>Error: {error}</p>;
  }

  return (
    <div>
      <h1>About</h1>
      {buildInfo ? Object.entries(buildInfo).map((info, index) => {
        return <div key={index}>
          <strong>{info[0]}:</strong> {(info[1] as any).toString()}
        </div>
      }) : <p>Loading build information...</p>}
    </div>
  );
}