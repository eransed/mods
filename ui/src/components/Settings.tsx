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
    camera_send_image: boolean;
    camera_send_image_resize_factor: number;
    opencv_display: boolean;
    skip_april_pose_estimation: boolean;
    angle_filter: number;
    min_decision_margin: number;
    device_index: number;
    device_width: number;
}

function configEqual(a: Config, b: Config): boolean {
    // return Object.is(a, b);
    if (a.http_port !== b.http_port) return false;
    if (a.ws_port !== b.ws_port) return false;
    if (a.log_level !== b.log_level) return false;
    if (a.allow_remote_connections !== b.allow_remote_connections) return false;
    if (a.enable_camera !== b.enable_camera) return false;
    if (a.camera_send_image !== b.camera_send_image) return false;
    if (a.camera_send_image_resize_factor !== b.camera_send_image_resize_factor) return false;
    if (a.opencv_display !== b.opencv_display) return false;
    if (a.skip_april_pose_estimation !== b.skip_april_pose_estimation) return false;
    if (a.angle_filter !== b.angle_filter) return false;
    if (a.min_decision_margin !== b.min_decision_margin) return false;
    if (a.device_index !== b.device_index) return false;
    if (a.device_width !== b.device_width) return false;
    return true;
}

export function Settings({ http_port }: SettingsProps) {
    let protocol = 'http'
    let url = `${protocol}://${window.location.hostname}:${http_port}`
    const [config, setConfig] = useState<Config | null>(null);
    const [configModified, setConfigModified] = useState<Config | null>(null);
    const [errState, setErrState] = useState<any>(null);

    // fetch the current configuration from the server
    useEffect(() => {
        const fetchConfig = async () => {
            try {
                const response = await fetch(`${url}/config`);
                if (response.ok) {
                    const config = await response.json();
                    setConfig(config);
                    setConfigModified(config);
                    console.log('Current configuration:', config);
                } else {
                    console.error('Failed to fetch configuration:', response.statusText);
                    setErrState(response.statusText)
                }
            } catch (error) {
                console.error('Error fetching configuration:', error);
                setErrState(error)
            }
        };

        fetchConfig();
    }, [http_port]);

    if (errState) {
        return <>
            <h1>Settings</h1>
            <p>{`${errState}`}</p>
        </>
    }

    return (
        <div>
            <h1>Settings</h1>
            {configModified && Object.entries(configModified).map(([key, value]) => (
                <div key={key} style={{ outline: 'none' }}>
                    <ConfigField
                        label={key}
                        value={value}
                        onChange={(newValue) => {
                            if (configModified) {
                                const updatedConfig = { ...configModified, [key]: newValue };
                                setConfigModified(updatedConfig);
                            }
                        }}
                    />
                </div>
            ))}
            {config && configModified &&
                <button className="config-save" disabled={configEqual(config, configModified)} onClick={() => {
                    if (configModified) {
                        setConfig(configModified)
                        updateConfig(url, configModified)
                    }
                }}>Save</button>
            }
        </div>
    );
}

interface ConfigFieldProps {
    label: string;
    value: boolean | number | string;
    onChange: (newValue: boolean | number | string) => void;
}

function ConfigField({ label, value, onChange }: ConfigFieldProps) {
    let inp = <div>Unsupported config parameter type: {typeof value} ({label})</div>
    let id = 'id-' + (1e9 * Math.random()).toFixed(0)
    if (typeof value === 'boolean') {
        inp = <input
            type="checkbox"
            checked={value}
            onChange={(e) => onChange(e.target.checked)}
            id={id}
        />
    } else if (typeof value === 'number') {
        let step = "1"
        if (value % 1 != 0) {
            step = "0.1"
        }
        inp = <input
            type="number"
            value={value}
            onChange={(e) => onChange(Number(e.target.value))}
            id={id}
            step={step}
        />
    } else if (typeof value === 'string') {
        inp = <input
            type="text"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            id={id}
        />
    }

    return (
        <div style={{}}>
            <div className="config-field">
                <label>
                    <b>
                        {label}
                    </b>
                    <i>
                        ({typeof value}):
                    </i>
                    {inp}
                    {typeof value === 'boolean' && <span></span>}
                </label>
            </div>
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
    }).then((response) => {
        if (!response.ok) {
            throw new Error(`Failed to update configuration: ${response.statusText}`);
        }
        console.log('Configuration updated successfully');
    }).catch((error) => {
        console.error('Error updating configuration:', error);
    });
}
