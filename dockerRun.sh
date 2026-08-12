#!/bin/bash
docker run -d --restart=always -p 8123:8123/tcp -p 8124:8124/tcp mods-runner:latest
