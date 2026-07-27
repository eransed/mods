
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
  image_width: number;
  detection_time_us: number;
}

export function Camera({ webSocket }: CameraProps) {
  const [imageData, setImageData] = useState<string>('');
  const [detectionTime, setDetectionTime] = useState<number>(0);
  const [tagCount, setTagCount] = useState<number>(0);

  useEffect(() => {
    const handleMessage = (event: MessageEvent) => {
      try {
        const message = JSON.parse(event.data) as RawImageDetection;
        setImageData(message.image_data_base64);
        setDetectionTime(message.detection_time_us);
        setTagCount(message.tags.length);
      } catch (error) {
        console.error('Failed to parse camera message:', error);
      }
    };

    webSocket.addEventListener('message', handleMessage);

    return () => {
      webSocket.removeEventListener('message', handleMessage);
    };
  }, [webSocket]);

  return (
    <div>
      {imageData ? (
        <div>
          <img
            src={`data:image/png;base64,${imageData}`}
            alt="Camera Feed"
            style={{
              maxWidth: '100%',
              height: 'auto',
            }}
          />
          <div style={{color: '#eee' }}>
            <p>Tags detected: {tagCount}</p>
            <p>Detection time: {(detectionTime / 1000).toFixed(1)}ms</p>
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
