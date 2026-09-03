export interface Config {
    general_config:        GeneralConfig;
    logging_config:        LoggingConfig;
    user_interface_config: UserInterfaceConfig;
    camera_configs:        CameraConfig[];
    open_protocol_configs: OpenProtocolConfig[];
    volumes:               Volume[];
}

export interface CameraConfig {
    name:                            StringProperty;
    enable_camera:                   BoolProperty;
    opencv_display:                  BoolProperty;
    angle_filter:                    NumberProperty;
    min_decision_margin:             NumberProperty;
    device_index:                    NumberProperty;
    device_width:                    NumberProperty;
    camera_fetch_delay_ms:           NumberProperty;
    camera_send_image:               BoolProperty;
    camera_send_image_resize_factor: NumberProperty;
}

export interface NumberProperty {
    value:              number;
    default_value:      number;
    allowed_values:     null;
    input_type:         null;
    added_version:      string;
    description:        string;
    hide:               boolean;
    deprecated_version: string;
}

export interface BoolProperty {
    value:              boolean;
    default_value:      boolean;
    allowed_values:     null;
    input_type:         null;
    added_version:      string;
    description:        string;
    hide:               boolean;
    deprecated_version: string;
}

export interface StringProperty {
    value:              string;
    default_value:      string;
    allowed_values:     string[] | null;
    input_type:         null | string;
    added_version:      string;
    description:        string;
    hide:               boolean;
    deprecated_version: string;
}

export interface GeneralConfig {
    _bool_property:           BoolProperty;
    _string_property:         StringProperty;
    _number_property:         NumberProperty;
    http_port:                NumberProperty;
    ws_port:                  NumberProperty;
    allow_remote_connections: BoolProperty;
}

export interface LoggingConfig {
    log_level:            StringProperty;
    max_lines_per_file:   NumberProperty;
    max_log_file_to_keep: NumberProperty;
}

export interface OpenProtocolConfig {
    activated:          BoolProperty;
    name:               StringProperty;
    ip:                 StringProperty;
    port:               NumberProperty;
    keep_alive_time_ms: NumberProperty;
    reconnect_delay_ms: NumberProperty;
    mid_0001_config:    Mid0001_Config;
}

export interface Mid0001_Config {
    rev:    number;
    active: boolean;
}

export interface UserInterfaceConfig {
    notification_position: StringProperty;
    background_color:      StringProperty;
    foreground_color:      StringProperty;
    accent_color:          StringProperty;
}

export interface Volume {
    name:              string;
    position:          Position;
    enter_radius:      number;
    exit_radius:       number;
    coordinate_system: string;
}

export interface Position {
    x: number;
    y: number;
    z: number;
}
