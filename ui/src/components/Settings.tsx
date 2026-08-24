import { useEffect, useId, useRef, useState } from "react";
import type { ReactNode } from "react";
import type { Config } from "../types/Config";
import { Button } from "./Button";

export type OpenProtocolState = {
    name: string;
    ip: string;
    port: number;
    connected: boolean;
    ping_ms: number | null;
    error: string | null;
};

export interface SettingsProps {
    http_port: number
    webSocket: WebSocket | null
    openProtocolStates: Record<string, OpenProtocolState>
}

function configEqual(a: Config, b: Config): boolean {
    return JSON.stringify(a) === JSON.stringify(b);
}

function configChangeCount(current: unknown, saved: unknown): number {
    if (isConfigProperty(current) && isConfigProperty(saved)) {
        return current.value === saved.value ? 0 : 1;
    }
    if (Array.isArray(current) && Array.isArray(saved)) {
        const length = Math.max(current.length, saved.length);
        return Array.from({ length }, (_, index) => configChangeCount(current[index], saved[index]))
            .reduce((total, changes) => total + changes, 0);
    }
    if (current !== null && typeof current === 'object' && saved !== null && typeof saved === 'object') {
        const keys = new Set([...Object.keys(current), ...Object.keys(saved)]);
        return [...keys].reduce((total, key) => total + configChangeCount(
            (current as ConfigObject)[key],
            (saved as ConfigObject)[key],
        ), 0);
    }
    return current === saved ? 0 : 1;
}

export function Settings({ http_port, webSocket, openProtocolStates: receivedOpenProtocolStates }: SettingsProps) {
    let protocol = 'http'
    let url = `${protocol}://${window.location.hostname}:${http_port}`
    const [config, setConfig] = useState<Config | null>(null);
    const [configModified, setConfigModified] = useState<Config | null>(null);
    const [defaultConfig, setDefaultConfig] = useState<Config | null>(null);
    const [openProtocolStates, setOpenProtocolStates] = useState<Record<string, OpenProtocolState>>(receivedOpenProtocolStates);
    const [connectingOpenProtocolNames, setConnectingOpenProtocolNames] = useState<Set<string>>(new Set());
    const [errState, setErrState] = useState<any>(null);
    const configFileInput = useRef<HTMLInputElement>(null);

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

    useEffect(() => {
        const receivedNames = new Set(Object.keys(receivedOpenProtocolStates));
        setConnectingOpenProtocolNames((current) => {
            const next = new Set([...current].filter((name) => !receivedNames.has(name)));
            return next.size === current.size ? current : next;
        });
        setOpenProtocolStates((current) => ({ ...current, ...receivedOpenProtocolStates }));
    }, [receivedOpenProtocolStates]);

    useEffect(() => {
        if (!webSocket) return;

        const handleMessage = (event: MessageEvent) => {
            try {
                const message = JSON.parse(event.data) as { OpenProtocolState?: OpenProtocolState };
                const state = message.OpenProtocolState;
                if (state) {
                    setConnectingOpenProtocolNames((current) => {
                        if (!current.has(state.name)) return current;
                        const next = new Set(current);
                        next.delete(state.name);
                        return next;
                    });
                    setOpenProtocolStates((current) => ({ ...current, [state.name]: state }));
                }
            } catch {
                // Ignore non-JSON messages intended for other UI features.
            }
        };

        webSocket.addEventListener('message', handleMessage);
        return () => webSocket.removeEventListener('message', handleMessage);
    }, [webSocket]);

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
                <h1>Settings{config && configModified && !configEqual(config, configModified) ? ' *' : ''}</h1>
                {config && configModified &&
                    <div className="settings-actions">
                        <Button type="button" onClick={() => downloadConfig(configModified)}>Download config</Button>
                        <Button type="button" onClick={() => configFileInput.current?.click()}>Upload config</Button>
                        <Button type="button" onClick={async () => {
                            try {
                                const resetConfigValue = await factoryResetConfig(url);
                                setConfig(resetConfigValue);
                                setConfigModified(resetConfigValue);
                                setDefaultConfig(resetConfigValue);
                                setConnectingOpenProtocolNames(new Set(resetConfigValue.open_protocol_configs.map((entry) => entry.name.value)));
                            } catch (error) {
                                setErrState(`Could not factory reset config: ${error}`);
                            }
                        }}>Factory reset</Button>
                        <input
                            ref={configFileInput}
                            className="settings-config-file-input"
                            type="file"
                            accept="application/json,.json"
                            onChange={async (event) => {
                                const file = event.target.files?.[0];
                                event.target.value = '';
                                if (!file) return;
                                try {
                                    const uploadedConfig = JSON.parse(await file.text()) as Config;
                                    setConfigModified(uploadedConfig);
                                } catch (error) {
                                    setErrState(`Could not upload config: ${error}`);
                                }
                            }}
                        />
                        {!configEqual(config, configModified) && <Button type="button" onClick={() => {
                            setConnectingOpenProtocolNames(new Set());
                            setConfigModified(config);
                        }}>
                            Undo {configChangeCount(configModified, config)} changes
                        </Button>}
                        <Button className="config-save" disabled={configEqual(config, configModified)} onClick={() => {
                            setConnectingOpenProtocolNames(new Set(configModified.open_protocol_configs.map((entry) => entry.name.value)));
                            setConfig(configModified)
                            updateConfig(url, configModified)
                        }}>Save</Button>
                    </div>
                }
            </div>
            {configModified && Object.entries(configModified).map(([key, value]) => (
                <ConfigSection
                    key={key}
                    label={key}
                    value={value}
                    oldValue={config ? config[key as keyof Config] : null}
                    defaultValue={defaultConfig ? defaultConfig[key as keyof Config] : null}
                    openProtocolStates={openProtocolStates}
                    connectingOpenProtocolNames={connectingOpenProtocolNames}
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
    label: ReactNode;
    value: unknown;
    oldValue: unknown;
    defaultValue?: unknown;
    openProtocolStates?: Record<string, OpenProtocolState>;
    connectingOpenProtocolNames?: Set<string>;
    onChange: (newValue: unknown) => void;
    onRemove?: () => void;
}

function ConfigSection({ label, value, oldValue, defaultValue, openProtocolStates = {}, connectingOpenProtocolNames = new Set(), onChange, onRemove }: ConfigSectionProps) {
    const sectionLabel = typeof label === 'string' ? label : '';
    const entries = Array.isArray(value)
        ? value.map((entry, index) => [String(index), entry] as const)
        : value !== null && typeof value === 'object'
            ? Object.entries(value)
            : [];

    return (
        <section className="settings-section">
            <h2>
                <span className="settings-section-heading-content">
                    {configSectionHeadingLabel(label)}
                    {Array.isArray(value) && <span className="settings-section-count">({value.length})</span>}
                </span>
                {onRemove && <Button className="settings-section-remove" type="button" onClick={onRemove}>Remove</Button>}
            </h2>
            {entries.map(([key, entry]) => (
                <ConfigEntry
                    key={key}
                    label={Array.isArray(value) ? arrayEntryLabel(
                        sectionLabel,
                        entry,
                        Array.isArray(oldValue) ? oldValue[Number(key)] : null,
                        Number(key),
                        openProtocolStates,
                        connectingOpenProtocolNames,
                    ) : key}
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
                    openProtocolStates={openProtocolStates}
                    connectingOpenProtocolNames={connectingOpenProtocolNames}
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
                <Button type="button" onClick={() => onChange([
                    ...value,
                    createArrayEntry(sectionLabel, value, defaultValue),
                ])}>
                    Add {configTypeLabel(sectionLabel)}
                </Button>
            )}
        </section>
    );
}

function configModuleLabel(label: string): string {
    // Keep server keys for unknown modules while presenting friendly known names.
    if (label === 'general_config') return 'General';
    if (label === 'logging_config') return 'Logging';
    if (label === 'camera_configs') return 'Camera Devices';
    if (label === 'open_protocol_configs') return 'OpenProtocol Devices';
    return label;
}

function configSectionHeadingLabel(label: ReactNode) {
    return typeof label === 'string' ? configModuleLabel(label) : label;
}

function arrayEntryLabel(
    sectionLabel: string,
    entry: unknown,
    oldEntry: unknown,
    index: number,
    openProtocolStates: Record<string, OpenProtocolState>,
    connectingOpenProtocolNames: Set<string>,
): ReactNode {
    // Use device names as collection entry labels when the server provides them.
    if (sectionLabel === 'camera_configs' || sectionLabel === 'open_protocol_configs') {
        if (entry !== null && typeof entry === 'object' && 'name' in entry) {
            const name = (entry as ConfigObject).name;
            if (isConfigProperty(name) && typeof name.value === 'string') {
                const oldName = oldEntry !== null && typeof oldEntry === 'object' && 'name' in oldEntry
                    ? configPropertyString((oldEntry as ConfigObject).name)
                    : undefined;
                const stateName = oldName && oldName !== name.value ? oldName : name.value;
                const isJustAdded = sectionLabel === 'open_protocol_configs' && oldEntry == null;
                const isConnecting = sectionLabel === 'open_protocol_configs' && connectingOpenProtocolNames.has(name.value);
                const state = !isJustAdded && sectionLabel === 'open_protocol_configs'
                    ? openProtocolStates[stateName]
                    : undefined;
                if (sectionLabel === 'open_protocol_configs') {
                    const ip = configPropertyString((entry as ConfigObject).ip) ?? 'unknown';
                    const port = configPropertyNumber((entry as ConfigObject).port) ?? 0;
                    const address = state ? `${state.ip}:${state.port}` : `${ip}:${port}`;
                    const status = isJustAdded
                        ? 'Just added'
                        : isConnecting
                            ? 'Connecting...'
                            : state?.connected
                                ? `Connected: Ping ${state.ping_ms ?? '-'} ms`
                                : state
                                    ? `Disconnected: '${state.error ?? 'unknown error'}'`
                                    : null;
                    return <>
                        <span className="settings-entry-address">
                            {(state || isJustAdded || isConnecting) && <span className={`dot ${isJustAdded || isConnecting ? 'dot-connecting' : state?.connected ? 'dot-connected' : 'dot-error'}`} aria-hidden="true" />}
                            <b className="settings-entry-name">{name.value}</b>
                            <span className="settings-entry-address-value">{address}</span>
                        </span>
                        <span className="settings-entry-connection">
                            {status && <span className="config-field-state">{status}</span>}
                        </span>
                    </>;
                }
                return name.value;
            }
        }
    }

    return `${index + 1}. ${sectionLabel}`;
}

function configPropertyString(value: unknown): string | undefined {
    return isConfigProperty(value) && typeof value.value === 'string' ? value.value : undefined;
}

function configPropertyNumber(value: unknown): number | undefined {
    return isConfigProperty(value) && typeof value.value === 'number' ? value.value : undefined;
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

function ConfigEntry({ label, value, oldValue, defaultValue, openProtocolStates = {}, connectingOpenProtocolNames = new Set(), onChange, onRemove }: ConfigSectionProps) {
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
                    : <ConfigSection label={label} value={value} oldValue={oldValue} defaultValue={defaultValue} openProtocolStates={openProtocolStates} connectingOpenProtocolNames={connectingOpenProtocolNames} onChange={onChange} onRemove={onRemove} />}
            </div>
        );
    }

    return null;
}

function createArrayEntry(sectionLabel: string, entries: unknown[], defaultValue: unknown): unknown {
    const entry = createDefaultArrayEntry(defaultValue);
    if (sectionLabel !== 'open_protocol_configs' || entry === null || typeof entry !== 'object') {
        return entry;
    }

    const name = configPropertyString((entry as ConfigObject).name);
    if (!name) return entry;

    const usedNames = new Set(entries.map((currentEntry) => {
        if (currentEntry !== null && typeof currentEntry === 'object') {
            return configPropertyString((currentEntry as ConfigObject).name);
        }
        return undefined;
    }).filter((currentName): currentName is string => currentName !== undefined));
    let index = 1;
    while (usedNames.has(`${name}_${index}`)) index += 1;

    const namedEntry = cloneConfigValue(entry) as ConfigObject;
    const nameProperty = namedEntry.name;
    if (isConfigProperty(nameProperty)) {
        namedEntry.name = { ...nameProperty, value: `${name}_${index}` };
    }
    return namedEntry;
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

function downloadConfig(config: Config): void {
    const blob = new Blob([JSON.stringify(config, null, 2)], { type: 'application/json' });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.download = 'config.json';
    link.click();
    URL.revokeObjectURL(link.href);
}

async function factoryResetConfig(url: string): Promise<Config> {
    const response = await fetch(`${url}/reset_config`);
    if (!response.ok) {
        throw new Error(`Failed to reset configuration: ${response.statusText}`);
    }
    return await response.json() as Config;
}

interface ConfigFieldProps {
    label: ReactNode;
    property: ConfigProperty;
    oldProperty: ConfigProperty | null;
    onChange: (newValue: unknown) => void;
}

function ConfigField({ label, property, oldProperty, onChange }: ConfigFieldProps) {
    const { value } = property;
    const id = useId()
    let input = <div style={{ color: '#e00' }}><b>Unsupported config parameter type: {typeof value}</b></div>
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
