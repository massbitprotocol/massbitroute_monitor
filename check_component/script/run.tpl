#!/bin/bash
cd /opt/verification

export RUST_LOG=info
export RUST_LOG_TYPE=console
export ZONE=[[ZONE]]
export BENCHMARK_WRK_PATH=./
export DOMAIN=massbitroute.[[ENV]]

./mbr-check-component check-kind -n https://portal.$DOMAIN/mbr/node/list/verify \
    -g https://portal.$DOMAIN/mbr/gateway/list/verify \
    -b base-endpoint.json \
    -c check-flow.json \
    --domain $DOMAIN

