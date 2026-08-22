export interface Config {
    http_port:                       number;
    ws_port:                         number;
    allow_remote_connections:        boolean;
    enable_camera:                   boolean;
    opencv_display:                  boolean;
    angle_filter:                    number;
    min_decision_margin:             number;
    device_index:                    number;
    device_width:                    number;
    camera_fetch_delay_ms:           number;
    camera_send_image:               boolean;
    camera_send_image_resize_factor: number;
    logging_config:                  LoggingConfig;
    open_protocol_config:            OpenProtocolConfig;
}

export interface LoggingConfig {
    log_level:          string;
    max_lines_per_file: number;
    max_files:          number;
}

export interface OpenProtocolConfig {
    open_protocol_clints: OpenProtocolClint[];
}

export interface OpenProtocolClint {
    name:               string;
    ip:                 string;
    port:               number;
    keep_alive_time_ms: number;
    reconnect_delay_ms: number;
    mid_0001_config:    Mid0001_Config;
}

export interface Mid0001_Config {
    rev:    number;
    active: boolean;
}
