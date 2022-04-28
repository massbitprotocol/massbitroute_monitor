#!/bin/bash

cd /opt/fisherman

export RUST_LOG=info
export RUST_LOG_TYPE=console
export ZONE=[[ZONE]]
export BENCHMARK_WRK_PATH=./

./mbr-fisherman run-fisherman -n https://portal.massbitroute.dev/mbr/node/list/verify -g https://portal.massbitroute.dev/mbr/gateway/list/verify -d https://dapi.massbit.io/deploy/info/dapi/listid -c check-flow.json -b base-endpoint.json -m  wss://chain.massbitroute.dev --signer-phrase "bottom drive obey lake curtain smoke basket hold race lonely fit walk"  --domain massbitroute.dev

