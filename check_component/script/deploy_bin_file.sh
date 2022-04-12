#!/bin/bash
cargo build --release
echo "Update bin file"
rsync -avz ../target/release/mbr-check-component mbr-verify:/opt/verification/mbr-check-component
rsync -avz ../src/archive/check-flow.json mbr-verify:/opt/verification/check-flow.json
rsync -avz ../src/archive/base-endpoint.json mbr-verify:/opt/verification/base-endpoint.json
rsync -avz ../config_check_component.json mbr-verify:/opt/check_component/config_check_component.json

echo "Update run script"
rsync -avz run.sh mbr-verify:/opt/verification/run.sh

echo "Restart service"
ssh mbr-verify < restart_verify_service.sh