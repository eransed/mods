import { useEffect, useId, useState } from "react";
import type { Config } from "../types/Config";
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
    const [defaultConfig, setDefaultConfig] = useState<Config | null>(null);
    const [errState, setErrState] = useState<any>(null);

    // fetch the current configuration from the server
    useEffect(() => {
        const fetchConfig = async () => {
            try {
                const [configResponse, defaultConfigResponse] = await Promise.all([
                    fetch(`${url}/config`),
                    fetch(`${url}/default_config`),
                ]);
                if (configResponse.ok && defaultConfigResponse.ok) {
                    // Load active and default values separately so new entries use server metadata.
                    const config = await configResponse.json() as Config;
                    const defaultConfig = await defaultConfigResponse.json() as Config;
                    setConfig(config);
                    setConfigModified(config);
                    setDefaultConfig(defaultConfig);
                    console.log('Current configuration:', config);
                } else {
                    const response = !configResponse.ok ? configResponse : defaultConfigResponse;
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
            {/* Keep the page identity and save action visible while settings scroll. */}
            <div className="settings-sub-top-bar">
                <h1>Settings</h1>
                {config && configModified &&
                    <Button className="config-save" disabled={configEqual(config, configModified)} onClick={() => {
                        if (configModified) {
                            setConfig(configModified)
                            updateConfig(url, configModified)
                        }
                    }}>Save</Button>
                }
            </div>
            {configModified && Object.entries(configModified).map(([key, value]) => (
                <ConfigSection
                    key={key}
                    label={key}
                    value={value}
                    oldValue={config ? config[key as keyof Config] : null}
                    defaultValue={defaultConfig ? defaultConfig[key as keyof Config] : null}
                    onChange={(newValue) => {
                        if (configModified) {
                            const updatedConfig = { ...configModified, [key]: newValue } as Config;
                            setConfigModified(updatedConfig);
                        }
                    }}
                />
            ))}
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
    defaultValue?: unknown;
    onChange: (newValue: unknown) => void;
    onRemove?: () => void;
}

function ConfigSection({ label, value, oldValue, defaultValue, onChange, onRemove }: ConfigSectionProps) {
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
                    defaultValue={Array.isArray(defaultValue)
                        ? defaultValue[Number(key)]
                        : defaultValue !== null && typeof defaultValue === 'object'
                            ? (defaultValue as ConfigObject)[key]
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
                <Button type="button" onClick={() => onChange([...value, createDefaultArrayEntry(defaultValue)])}>
                    Add {configTypeLabel(label)}
                </Button>
            )}
        </section>
    );
}

function configTypeLabel(label: string): string {
    // Name each configurable device type explicitly in its add action.
    if (label === 'open_protocol_configs') return 'OpenProtocol Device';
    if (label === 'camera_configs') return 'Camera Device';

    // Turn any future collection names into readable labels by default.
    return label
        .replace(/_configs$/, '')
        .replace(/(^|_)\w/g, (match) => match.replace('_', '').toUpperCase());
}

function ConfigEntry({ label, value, oldValue, defaultValue, onChange, onRemove }: ConfigSectionProps) {
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
                    : <ConfigSection label={label} value={value} oldValue={oldValue} defaultValue={defaultValue} onChange={onChange} onRemove={onRemove} />}
            </div>
        );
    }

    return null;
}

function createDefaultArrayEntry(value: unknown): unknown {
    // Clone the server-provided entry without changing its configured default values.
    return Array.isArray(value) && value.length > 0 ? cloneConfigValue(value[0]) : {};
}

function cloneConfigValue(value: unknown): unknown {
    // Deep-copy nested config objects so editing a new entry cannot mutate defaults.
    if (Array.isArray(value)) return value.map(cloneConfigValue);
    if (value !== null && typeof value === 'object') {
        return Object.fromEntries(Object.entries(value).map(([key, child]) => [key, cloneConfigValue(child)]));
    }
    return value;
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
            {/* Keep the editable setting identity together in the row's top section. */}
            <div className="config-field-top">
                <div className="config-field-name"><b>{label}</b></div>
                <div className="config-field-value">
                    {typeof value === 'boolean' ? (
                        /* Associate the visible custom control with its hidden input. */
                        <label className="config-checkbox" htmlFor={id}>
                            {input}
                            <span className="checkbox"></span>
                        </label>
                    ) : input}
                    {oldProperty && value !== oldProperty.value && (
                        <span className="config-field-old-value">Previous: {`${oldProperty.value}`}</span>
                    )}
                </div>
            </div>
            {/* Keep descriptive metadata together below the editable controls. */}
            <div className="config-field-bottom">
                <div className="config-field-type">Type: {typeof value}</div>
                <div className="config-field-description">{property.description}</div>
                <div className="config-field-version">Added {property.added_version}</div>
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
