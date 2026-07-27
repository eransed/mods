import { useEffect, useState } from "react";

export interface SettingsProps {
    http_port: number
}

export interface Config {
    http_port: number;
    ws_port: number;
    log_level: string;
    allow_remote_connections: boolean;
    enable_camera: boolean;
    opencv_display: boolean;
    skip_april_pose_estimation: boolean;
    angle_filter: number;
    min_decision_margin: number;
    device_index: number;
    device_width: number;
}

export function Settings({ http_port }: SettingsProps) {
    let protocol = 'http'
    let url = `${protocol}://${window.location.hostname}:${http_port}`
    const [config, setConfig] = useState<Config | null>(null);

    // fetch the current configuration from the server
    useEffect(() => {
        const fetchConfig = async () => {
            try {
                const response = await fetch(`${url}/config`);
                if (response.ok) {
                    const config = await response.json();
                    setConfig(config);
                    console.log('Current configuration:', config);
                } else {
                    console.error('Failed to fetch configuration:', response.statusText);
                }
            } catch (error) {
                console.error('Error fetching configuration:', error);
            }
        };

        fetchConfig();
    }, [http_port]);

    return (
        <div>
            <h1>Settings</h1>
            {config && Object.entries(config).map(([key, value]) => (
                <div key={key}>
                    {/* <strong>{key}:</strong> {String(value)} */}
                    {typeof value === 'boolean' && (
                        <ConfigBooleanField
                            label={key}
                            value={value}
                            onChange={(newValue) => {
                                if (config) {
                                    const updatedConfig = { ...config, [key]: newValue };
                                    setConfig(updatedConfig);
                                    updateConfig(url, updatedConfig);
                                }
                            }}
                        />
                    )}

                    {typeof value === 'number' && (
                        <ConfigNumberField
                            label={key}
                            value={value}
                            onChange={(newValue) => {
                                if (config) {
                                    const updatedConfig = { ...config, [key]: newValue };
                                    setConfig(updatedConfig);
                                    updateConfig(url, updatedConfig);
                                }
                            }}
                        />
                    )}

                    {typeof value === 'string' && (
                        <ConfigStringField
                            label={key}
                            value={value}
                            onChange={(newValue) => {
                                if (config) {
                                    const updatedConfig = { ...config, [key]: newValue };
                                    setConfig(updatedConfig);
                                    updateConfig(url, updatedConfig);
                                }
                            }}
                        />
                    )}

                </div>
            ))}
        </div>
    );
}

function ConfigBooleanField({ label, value, onChange }: { label: string; value: boolean; onChange: (newValue: boolean) => void }) {
    return (
        <div>
            <label>
                {label}:
                <input
                    type="checkbox"
                    checked={value}
                    onChange={(e) => onChange(e.target.checked)}
                />
            </label>
        </div>
    );
}

function ConfigNumberField({ label, value, onChange }: { label: string; value: number; onChange: (newValue: number) => void }) {
    return (
        <div>
            <label>
                {label}:
                <input
                    type="number"
                    value={value}
                    onChange={(e) => onChange(Number(e.target.value))}
                />
            </label>
        </div>
    );
}

function ConfigStringField({ label, value, onChange }: { label: string; value: string; onChange: (newValue: string) => void }) {
    return (
        <div>
            <label>
                {label}:
                <input
                    type="text"
                    value={value}
                    onChange={(e) => onChange(e.target.value)}
                />
            </label>
        </div>
    );
}

function updateConfig(url: string, newConfig: Config) {
    fetch(`${url}/set_config`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(newConfig),
    })
        .then((response) => {
            if (!response.ok) {
                throw new Error(`Failed to update configuration: ${response.statusText}`);
            }
            console.log('Configuration updated successfully');
        })
        .catch((error) => {
            console.error('Error updating configuration:', error);
        }
        );
}
