import { useEffect, useId, useState } from "react";
import type { Config, OpenProtocolConfig } from "../types/Config";
import { Button } from "./Button";

export interface SettingsProps {
    http_port: number
}

function configEqual(a: Config, b: Config): boolean {
    return JSON.stringify(a) === JSON.stringify(b);
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
    }, [url]);

    if (errState) {
        return <>
            <h1>Settings</h1>
            <p>{`${errState}`}</p>
        </>
    }

    return (
        <div className="settings-page">
            <h1>Settings</h1>
            {configModified && Object.entries(configModified).map(([key, value]) => (
                <ConfigSection
                    key={key}
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

type ConfigObject = Record<string, unknown>;
type ConfigProperty = {
    value: boolean | number | string;
    default_value: boolean | number | string;
    added_version: string;
    description: string;
    hide: boolean;
    deprecated_version: string;
};

function isConfigProperty(value: unknown): value is ConfigProperty {
    return value !== null && typeof value === 'object' && 'value' in value
        && 'default_value' in value && 'added_version' in value
        && 'description' in value && 'hide' in value;
}

interface ConfigSectionProps {
    label: string;
    value: unknown;
    oldValue: unknown;
    onChange: (newValue: unknown) => void;
    onRemove?: () => void;
}

function ConfigSection({ label, value, oldValue, onChange, onRemove }: ConfigSectionProps) {
    const entries = Array.isArray(value)
        ? value.map((entry, index) => [String(index), entry] as const)
        : value !== null && typeof value === 'object'
            ? Object.entries(value)
            : [];

    return (
        <section className="settings-section">
            <h2>{label} {onRemove && <Button type="button" onClick={onRemove}>Remove</Button>}</h2>
            {entries.map(([key, entry]) => (
                <ConfigEntry
                    key={key}
                    label={Array.isArray(value) ? `${label} ${Number(key) + 1}` : key}
                    value={entry}
                    oldValue={Array.isArray(oldValue)
                        ? oldValue[Number(key)]
                        : oldValue !== null && typeof oldValue === 'object'
                            ? (oldValue as ConfigObject)[key]
                            : null}
                    onChange={(newValue) => {
                        if (Array.isArray(value)) {
                            const updatedValue = [...value];
                            updatedValue[Number(key)] = newValue;
                            onChange(updatedValue);
                        } else {
                            onChange({ ...(value as ConfigObject), [key]: newValue });
                        }
                    }}
                    onRemove={Array.isArray(value) ? () => {
                        const updatedValue = [...value];
                        updatedValue.splice(Number(key), 1);
                        onChange(updatedValue);
                    } : undefined}
                />
            ))}
            {Array.isArray(value) && (
                <Button type="button" onClick={() => onChange([...value, createDefaultArrayEntry(label, value)])}>Add</Button>
            )}
        </section>
    );
}

function ConfigEntry({ label, value, oldValue, onChange, onRemove }: ConfigSectionProps) {
    if (isConfigProperty(value)) {
        return value.hide ? null : (
            <ConfigField
                label={label}
                property={value}
                oldProperty={isConfigProperty(oldValue) ? oldValue : null}
                onChange={(newValue) => onChange({ ...value, value: newValue })}
            />
        );
    }

    if (Array.isArray(value) || (value !== null && typeof value === 'object')) {
        return (
            <div className="settings-subsection">
                {Array.isArray(value) && value.length === 0
                    ? <p>No entries</p>
                    : <ConfigSection label={label} value={value} oldValue={oldValue} onChange={onChange} onRemove={onRemove} />}
            </div>
        );
    }

    return null;
}

function createDefaultArrayEntry(label: string, entries: unknown[]): OpenProtocolConfig | unknown {
    if (label === 'open_protocol_configs') {
        return {
            activated: { value: true, default_value: true, added_version: '1.0.0', description: 'Whether the client is activated', hide: false, deprecated_version: '' },
            name: { value: 'default', default_value: 'default', added_version: '1.0.0', description: 'The name of the client', hide: false, deprecated_version: '' },
            ip: { value: '127.0.0.1', default_value: '127.0.0.1', added_version: '1.0.0', description: 'The IP address of the client', hide: false, deprecated_version: '' },
            port: { value: 4545, default_value: 4545, added_version: '1.0.0', description: 'The port of the client', hide: false, deprecated_version: '' },
            keep_alive_time_ms: { value: 7500, default_value: 7500, added_version: '1.0.0', description: 'The keep-alive time in milliseconds', hide: false, deprecated_version: '' },
            reconnect_delay_ms: { value: 5000, default_value: 5000, added_version: '1.0.0', description: 'The reconnect delay in milliseconds', hide: false, deprecated_version: '' },
            mid_0001_config: {
                rev: 6,
                active: true,
            },
        };
    }

    return createDefaultValue(entries[0]);
}

function createDefaultValue(value: unknown): unknown {
    // Preserve property metadata when creating a new array entry.
    if (isConfigProperty(value)) {
        return { ...value, value: createDefaultValue(value.value) };
    }
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
    property: ConfigProperty;
    oldProperty: ConfigProperty | null;
    onChange: (newValue: unknown) => void;
}

function ConfigField({ label, property, oldProperty, onChange }: ConfigFieldProps) {
    const { value } = property;
    const id = useId()
    let input = <div style={{ color: '#e00' }}><b>Unsupported config parameter type: {typeof value} ({label})</b></div>
    if (typeof value === 'boolean') {
        input = <input
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
        input = <input
            type="number"
            value={value}
            onChange={(e) => onChange(Number(e.target.value))}
            id={id}
            step={step}
        />
    } else if (typeof value === 'string') {
        input = <input
            type="text"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            id={id}
        />
    }

    return (
        <div className="config-field">
            <div className="config-field-name"><b>{label}</b></div>
            <div className="config-field-value">
                {input}
                {typeof value === 'boolean' && <span className="checkbox"></span>}
                {oldProperty && value !== oldProperty.value && (
                    <span className="config-field-old-value">Previous: {`${oldProperty.value}`}</span>
                )}
            </div>
            <div className="config-field-type">{typeof value}</div>
            <div className="config-field-description">{property.description}</div>
            <div className="config-field-version">Added {property.added_version}</div>
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
