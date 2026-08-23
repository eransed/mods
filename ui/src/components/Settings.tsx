import { useEffect, useState } from "react";
import type { Config, OpenProtocolClient } from "../types/Config";
import { Button } from "./Button";

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
                                const updatedConfig = { ...configModified, [key]: newValue } as Config;
                                setConfigModified(updatedConfig);
                            }
                        }}
                    />
                </div>
            ))}
            {config && configModified &&
                <Button className="config-save" disabled={configEqual(config, configModified)} onClick={() => {
                    if (configModified) {
                        setConfig(configModified)
                        updateConfig(url, configModified)
                    }
                }}>Save</Button>
            }
        </div>
    );
}

interface ObjectConfigField {
    value: Record<string, unknown> | unknown[],
    oldValue: Record<string, unknown> | unknown[] | null,
    onChange: (newValue: Record<string, unknown> | unknown[]) => void;
}

function objectConfigField(conf: ObjectConfigField) {
    const isArray = Array.isArray(conf.value);
    return (
        <div>
            {Object.entries(conf.value).map(([key, value]) => (
                <div key={key}>
                    <ConfigField
                        label={key}
                        value={value}
                        oldValue={conf.oldValue === null
                            ? null
                            : Array.isArray(conf.oldValue)
                                ? conf.oldValue[Number(key)] ?? null
                                : conf.oldValue[key] ?? null}
                        onChange={(newValue) => {
                            const updatedValue = Array.isArray(conf.value) ? [...conf.value] : { ...conf.value };
                            if (Array.isArray(updatedValue)) {
                                updatedValue[Number(key)] = newValue;
                            } else {
                                updatedValue[key] = newValue;
                            }
                            conf.onChange(updatedValue);
                        }}
                        onRemove={isArray ? () => {
                            const updatedValue = [...conf.value as unknown[]];
                            updatedValue.splice(Number(key), 1);
                            conf.onChange(updatedValue);
                        } : undefined}
                    />
                </div>
            ))}
        </div>
    );
}

function createDefaultArrayEntry(label: string, entries: unknown[]): OpenProtocolClient | unknown {
    if (label === 'open_protocol_clients') {
        return {
            activated: true,
            name: 'default',
            ip: '127.0.0.1',
            port: 4545,
            keep_alive_time_ms: 7500,
            reconnect_delay_ms: 5000,
            mid_0001_config: {
                rev: 6,
                active: true,
            },
        };
    }

    return createDefaultValue(entries[0]);
}

function createDefaultValue(value: unknown): unknown {
    if (typeof value === 'boolean') return false;
    if (typeof value === 'number') return 0;
    if (typeof value === 'string') return '';
    if (Array.isArray(value)) return [];
    if (value !== null && typeof value === 'object') {
        return Object.fromEntries(Object.entries(value).map(([key, child]) => [key, createDefaultValue(child)]));
    }
    return {};
}

interface ConfigFieldProps {
    label: string;
    value: unknown;
    oldValue: unknown;
    onChange: (newValue: unknown) => void;
    onRemove?: () => void;
}

function ConfigField({ label, value, oldValue, onChange, onRemove }: ConfigFieldProps) {
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
    } else if (value !== null && typeof value === 'object') {
        inp = objectConfigField({
            value: value as Record<string, unknown> | unknown[],
            oldValue: oldValue as Record<string, unknown> | unknown[] | null,
            onChange,
        })
    }

    return (
        <div style={{}}>
            <div className="config-field">
                <label>
                    <b>
                        {label}
                    </b>
                    {onRemove && <Button type="button" onClick={onRemove}>-</Button>}
                    {value instanceof Array && <Button type="button" onClick={() => {
                        const entries = value as unknown[];
                        onChange([...entries, createDefaultArrayEntry(label, entries)]);
                    }}>+</Button>}
                    <i>
                        ({typeof value}):
                    </i>
                    {inp}
                    {typeof value === 'boolean' && <span className="checkbox"></span>}
                </label>
                {typeof value !== 'object' && (
                    <span className="config-field-old-value">
                        {value !== oldValue ? `${oldValue}` : null}
                    </span>
                )}
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
