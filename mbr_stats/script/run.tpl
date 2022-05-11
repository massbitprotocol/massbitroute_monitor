#!/bin/bash

cd /opt/stats

export RUST_LOG=info
export RUST_LOG_TYPE=console
export ZONE=[[ZONE]]

./mbr-stats update-stats --prometheus-url https://stat.mbr.massbitroute.dev/__internal_prometheus_ -m  wss://chain.massbitroute.dev:443 --list-project-url https://portal.massbitroute.dev/mbr/d-apis/project/list/verify

