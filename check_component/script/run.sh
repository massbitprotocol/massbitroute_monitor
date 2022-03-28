#!/bin/bash
cd /opt/verification
RUST_LOG=info RUST_LOG_TYPE=file
./mbr-check-component check-kind -n https://dapi.massbitroute.dev/deploy/info/node/listid -g https://dapi.massbitroute.dev/deploy/info/gateway/listid -d https://dapi.massbitroute.dev/deploy/info/dapi/listid -u https://dapi.massbitroute.dev/deploy/info/user/listid -b base-endpoint.json -c check-flow.json --domain massbitroute.dev

