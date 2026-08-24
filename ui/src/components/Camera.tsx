
import { useEffect, useState } from 'react';

export interface CameraProps {
    webSocket: WebSocket;
}

interface RawImageDetection {
    tags: Array<{
        id: number;
        center_image: [number, number];
        decision_margin: number;
        translation: [number, number, number];
        rotation: [number, number, number];
        pose_estimation_time_us: number;
    }>;
    image_data_base64: string;
    image_size: [number, number];
    native_image_size: [number, number];
    detection_time_us: number;
    image_encoding_time_us: number;
    send_freq: number;
}

export function Camera({ webSocket }: CameraProps) {
    const initialMessage = 'No data received'
    const [data, setData] = useState<RawImageDetection | null>(null);
    const [errorState, setErrorState] = useState<any>(initialMessage);
    const [bytesReceived, setBytesReceived] = useState<number>(0);
    const [framesReceived, setFramesReceived] = useState<number>(0);
    const [receivedFrequency, setReceivedFrequency] = useState<number>(0);
    const [msSinceFirstFrame, setMsSinceFirstFrame] = useState<number>(0);
    const [averageBytesPerFrame, setAverageBytesPerFrame] = useState<number>(0);
    let firstFrameTime: number | null = null;
    let framesReceivedCount = 0;
    let bytesReceivedCount = 0;

    useEffect(() => {
        const handleMessage = (event: MessageEvent) => {
            try {
                const message = JSON.parse(event.data) as RawImageDetection;
                if (message && message.tags) {
                    setData(message);
                    bytesReceivedCount += message.image_data_base64.length;
                    setBytesReceived(bytesReceivedCount);
                    framesReceivedCount += 1;
                    setFramesReceived(framesReceivedCount);
                    setAverageBytesPerFrame(bytesReceivedCount / framesReceivedCount);
                    setErrorState(null);
                    if (firstFrameTime === null) {
                        firstFrameTime = performance.now();
                    } else {
                        const msElapsed = performance.now() - firstFrameTime;
                        const frequency = (framesReceivedCount) / (msElapsed / 1000);
                        setMsSinceFirstFrame(msElapsed);
                        setReceivedFrequency(frequency);
                    }
                }
            } catch (error) {
                console.error('Failed to parse camera message:', error);
                setErrorState(`Failed to parse camera message: ${error}`)
            }
        };

        webSocket.addEventListener('message', handleMessage);

        return () => {
            webSocket.removeEventListener('message', handleMessage);
        };
    }, [webSocket]);

    if (errorState) {
        return <>
            <h1>Camera</h1>
            <p>{`${errorState}`}</p>
        </>
    }

    const dec = 2;
    const translationScale = 100; // Scale translation values by 100 for display

    return (
        <div>
            {data ? (
                <div>
                    {data.image_data_base64.length > 0 && <img
                        src={`data:image/png;base64,${data.image_data_base64}`}
                        alt="Camera Feed"
                        style={{
                            maxWidth: '100%',
                            height: 'auto',
                        }}
                    />}
                    <div style={{ color: '#eee' }}>
                        <p>Tags detected: {data.tags.length}</p>
                        <p>Detection time: {(data.detection_time_us / 1000).toFixed(dec)}ms</p>
                        <p>Image size: {data.image_size[0]} x {data.image_size[1]}</p>
                        <p>Native image size: {data.native_image_size[0]} x {data.native_image_size[1]}</p>
                        <p>Encoding time: {(data.image_encoding_time_us / 1000).toFixed(dec)}ms</p>
                        <p>Send frequency: {data.send_freq.toFixed(dec)}Hz</p>
                        <p>Received frequency: {receivedFrequency.toFixed(dec)}Hz</p>
                        <p>Data received: {(bytesReceived / 1024 / 1024).toFixed(dec)} MB</p>
                        <p>Average bytes per frame: {(averageBytesPerFrame / 1024 / 1024).toFixed(dec)} MB</p>
                        <p>Frames received: {framesReceived}</p>
                        <p>Time elapsed: {(msSinceFirstFrame / 1000).toFixed(dec)}s</p>
                        {data.tags.map((tag) => (
                            <div key={tag.id} style={{ marginBottom: '1rem', paddingLeft: '1rem', borderLeft: '2px solid #555', color: '#aea' }}>
                                <p>Tag ID: {tag.id}</p>
                                <p>Center Image: ({tag.center_image[0].toFixed(dec)}, {tag.center_image[1].toFixed(dec)})</p>
                                <p>Decision Margin: {tag.decision_margin.toFixed(dec)}</p>
                                <p>Translation: ({(translationScale * tag.translation[0]).toFixed(dec)}, {(translationScale * tag.translation[1]).toFixed(dec)}, {(translationScale * tag.translation[2]).toFixed(dec)})</p>
                                <p>Rotation: ({tag.rotation[0].toFixed(dec)}&deg;, {tag.rotation[1].toFixed(dec)}&deg;, {tag.rotation[2].toFixed(dec)}&deg;)</p>
                                <p>Pose Estimation Time: {(tag.pose_estimation_time_us / 1000).toFixed(dec)}ms</p>
                            </div>
                        ))}
                    </div>
                </div>
            ) : (
                <div style={{ padding: '2rem', textAlign: 'center', color: '#999' }}>
                    Waiting for camera feed...
                </div>
            )}
        </div>
    );
}
