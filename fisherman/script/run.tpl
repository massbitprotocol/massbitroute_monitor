#!/bin/bash

cd /opt/fisherman

export RUST_LOG=info
export RUST_LOG_TYPE=console
export ZONE=[[ZONE]]
export BENCHMARK_WRK_PATH=./
export DOMAIN=massbitroute.[[ENV]]
./mbr-fisherman run-fisherman -n https://portal.$DOMAIN/mbr/node/list/verify -g https://portal.$DOMAIN/mbr/gateway/list/verify -c check-flow.json -b base-endpoint.json -m  wss://chain.$DOMAIN --domain $DOMAIN --no_report_mode

