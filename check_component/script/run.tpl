#!/bin/bash
cd /opt/verification
ZONE=[[ZONE]] RUST_LOG=info RUST_LOG_TYPE=file BENCHMARK_WRK_PATH=./
./mbr-check-component check-kind -n https://portal.massbitroute.dev/mbr/node/list/verify \
    -g https://portal.massbitroute.dev/mbr/gateway/list/verify \
    -b base-endpoint.json \
    -c check-flow.json \
    --domain massbitroute.dev

