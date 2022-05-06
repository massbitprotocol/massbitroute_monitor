#!/bin/bash
cd /opt/stats
RUST_LOG=info RUST_LOG_TYPE=file
./mbr-stats update-stats --prometheus-gateway-url https://stat.mbr.massbitroute.dev/__internal_prometheus_gw --mvp-url wss://chain.massbitroute.dev:443 --signer-phrase "bottom drive obey lake curtain smoke basket hold race lonely fit walk" --list-project-url https://portal.massbitroute.dev/mbr/d-apis/project/list/verify

