#!/usr/bin/env bash

set -euo pipefail

curl http://127.0.0.1:8080/send
curl http://127.0.0.1:8080/send
curl http://127.0.0.1:8080/send
curl http://127.0.0.1:8080/send
printf "\n"

curl http://127.0.0.1:8080/config
printf "\n"

curl -X POST http://127.0.0.1:8080/set_config \
  -H 'Content-Type: application/json' \
  -d '{"http_port":8080,"ws_port":8085}'
printf "\n"

curl http://127.0.0.1:8080/config
printf "\n"

curl http://127.0.0.1:8080/shutdown
printf "\n"
