#!/bin/bash
export RED='\e[1;31m%-6s\e[m\n'
export GRN='\e[1;32m%-6s\e[m\n'
export YEL='\e[1;33m%-6s\e[m\n'
export BLU='\e[1;34m%-6s\e[m\n'
export MAG='\e[1;35m%-6s\e[m\n'
export CYN='\e[1;36m%-6s\e[m\n'

log_info() {
    dateTime=$(date '+%Y-%m-%d %H:%M:%S')
    printf "${MAG}" "INFO ${dateTime} $@"
}

log_note() {
    dateTime=$(date '+%Y-%m-%d %H:%M:%S')
    printf "${BLU}" "NOTE ${dateTime} $@"
}

log_ok() {
    dateTime=$(date '+%Y-%m-%d %H:%M:%S')
    printf "${GRN}" "OK ${dateTime} $@"
}

log_warn() {
    dateTime=$(date '+%Y-%m-%d %H:%M:%S')
    printf "${YEL}" "WARN ${dateTime} $@"
}

log_err() {
    dateTime=$(date '+%Y-%m-%d %H:%M:%S')
    printf "${RED}" "ERROR ${dateTime} $@"
}

if ! command -v systemctl >/dev/null 2>&1
then
  log_err "systemd not found"
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1
then
  log_err "cargo not found"
  exit 1
fi

if ! command -v node >/dev/null 2>&1
then
  log_err "node not found"
  exit 1
fi

if ! command -v npm >/dev/null 2>&1
then
  log_err "npm not found"
  exit 1
fi

if ! command -v opencv_version >/dev/null 2>&1
then
  log_err "opencv_version not found"
  exit 1
fi

start_time="$(date -u +%s)"

log_info "Building mods..."
cargo build --release --all-features ${cargo_extra_args}
rv=$?

printf "\n"
if [ $rv -ne 0 ]; then
  log_err "Build Failed"
  exit 1
else
  log_ok "Build Ok"
fi

end_time="$(date -u +%s)"
elapsed="$(($end_time-$start_time))"
log_note "Total build time: ${elapsed} seconds"

printf "\n"

log_info "Installing..."

log_info "Stopping service..."
sudo systemctl stop mods
sleep 3

log_info "Copying binary..."
sudo cp -v ./target/release/mods /usr/bin/mods

log_info "Creating config directory..."
sudo mkdir -p /etc/mods_service

log_info "Copying systemd unit file..."

sudo cp -v ./mods.service /etc/systemd/system/.

log_info "Enable mods systemd service..."
sudo systemctl enable mods

log_info "Starting mods systemd service..."
sudo systemctl start mods

sleep 3

sudo systemctl status -l --no-pager mods

printf "\n"

systemctl is-active --quiet mods

rv=$?
if [ $rv -ne 0 ]; then
  log_err "Installation failed with exit code ${rv}"
else
  log_ok "Installation was successful"
fi

end_time="$(date -u +%s)"
elapsed="$(($end_time-$start_time))"
log_note "Total install time: ${elapsed} seconds"

exit $rv
