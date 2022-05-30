#!/bin/bash

cd /opt/stats

export RUST_LOG=info
export RUST_LOG_TYPE=console
export ZONE=[[ZONE]]
export DOMAIN=massbitroute.[[ENV]]

./mbr-stats update-stats --prometheus-url https://stat.mbr.$DOMAIN/__internal_prometheus_ -m  wss://chain.$DOMAIN:443 --list-project-url https://portal.$DOMAIN/mbr/d-apis/project/list/verify

