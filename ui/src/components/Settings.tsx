import { useEffect, useState } from "react";
import type { Config, HTTPPort, LoggingConfig, OpenProtocolConfig } from "../types/Config";

export interface SettingsProps {
    http_port: number
}

function configEqual(a: Config, b: Config): boolean {
    for (let key of Object.keys(a)) {
        let av = a[key as keyof Config]
        let bv = b[key as keyof Config]
        if (av !== bv) {
            // console.log(`${key} is different: a['${key}']: ${av} !== b['${key}']: ${bv}`)
            return false;
        }
    }
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
                        oldValue={config ? config[key as keyof Config] : null}
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

interface ObjectConfigField {
    value: object,
    oldValue: object | null,
    onChange: (newValue: object) => void;
}

function objectConfigField(conf: ObjectConfigField) {
    return (
        <div>
            {Object.entries(conf.value).map(([key, value]) => (
                <div key={key}>
                    <ConfigField
                        label={key}
                        value={value}
                        oldValue={null}
                        onChange={(newValue) => {
                            console.log(`new value for key ${key}: ${newValue}`)
                        }}
                    />
                </div>
            ))}
        </div>
    );
}

interface ConfigFieldProps {
    label: string;
    value: boolean | number | string;
    oldValue: number | boolean | LoggingConfig | OpenProtocolConfig | HTTPPort | null
    onChange: (newValue: boolean | number | string | LoggingConfig | OpenProtocolConfig) => void;
}

function ConfigField({ label, value, oldValue, onChange }: ConfigFieldProps) {
    let inp = <div style={{ color: '#e00' }}><b>Unsupported config parameter type: {typeof value} ({label})</b></div>
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
    } else if (typeof value === 'object') {
        inp = objectConfigField({
            value: value,
            oldValue: null,
            onChange: function (newValue: object): void {
                console.log(`new value: {${newValue}}`)
            }
        })
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
                <span className="config-field-old-value">
                    {value !== oldValue ? `${oldValue}` : null}
                </span>
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
