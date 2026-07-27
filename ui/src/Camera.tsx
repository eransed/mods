
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
}

export function Camera({ webSocket }: CameraProps) {
    const [data, setData] = useState<RawImageDetection | null>(null);

    useEffect(() => {
        const handleMessage = (event: MessageEvent) => {
            try {
                const message = JSON.parse(event.data) as RawImageDetection;
                setData(message);
            } catch (error) {
                console.error('Failed to parse camera message:', error);
            }
        };

        webSocket.addEventListener('message', handleMessage);

        return () => {
            webSocket.removeEventListener('message', handleMessage);
        };
    }, [webSocket]);

    const dec = 1;
    const translationScale = 100; // Scale translation values by 100 for display

    return (
        <div>
            {data ? (
                <div>
                    <img
                        src={`data:image/png;base64,${data.image_data_base64}`}
                        alt="Camera Feed"
                        style={{
                            maxWidth: '100%',
                            height: 'auto',
                        }}
                    />
                    <div style={{ color: '#eee' }}>
                        <p>Tags detected: {data.tags.length}</p>
                        <p>Detection time: {(data.detection_time_us / 1000).toFixed(dec)}ms</p>
                        <p>Image size: {data.image_size[0]} x {data.image_size[1]}</p>
                        <p>Native image size: {data.native_image_size[0]} x {data.native_image_size[1]}</p>
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
