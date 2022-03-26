#!/bin/bash
cargo build --release
echo "Update bin file"
rsync -avz ../target/release/mbr-fisherman mbr-verify:/opt/fisherman/mbr-fisherman
rsync -avz ../../check_component/src/archive/check-flow.json mbr-verify:/opt/fisherman/check-flow.json
rsync -avz ../../check_component/src/archive/base-endpoint.json mbr-verify:/opt/fisherman/base-endpoint.json

echo "Update run script"
rsync -avz run.sh mbr-verify:/opt/fisherman/run.sh

echo "Restart service"
ssh mbr-verify < restart_service.sh