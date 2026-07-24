export interface BuildInfo {
    binary_release_size_kb: number;
    binary_debug_size_kb:   number;
    index_html_size_kb:     number;
    main_js_size_kb:        number;
    main_css_size_kb:       number;
    cargo_pkg_name:         string;
    cargo_pkg_version:      string;
    git_branch:             string;
    git_hash:               string;
    git_date:               Date;
    build_time_utc:         string;
    build_type:             string;
    build_uname:            string;
    rustc_version:          string;
    git_version:            string;
    docker_version:         string;
    node_version:           string;
    npm_version:            string;
    quicktype_version:      string;
    opencv_version:         string;
    target_arch:            string;
    target_avx2:            boolean;
    target_neon:            boolean;
    windows:                boolean;
}
