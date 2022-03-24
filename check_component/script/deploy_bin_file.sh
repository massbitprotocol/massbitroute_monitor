#!/bin/bash
cargo build --release
echo "Update bin file"
rsync -avz ../target/release/mbr-check-component mbr-verify:/opt/verification/mbr-check-component
rsync -avz ../target/release/check-flow.json mbr-verify:/opt/verification/check-flow.json
rsync -avz ../target/release/base-endpoint.json mbr-verify:/opt/verification/base-endpoint.json

echo "Update run script"
rsync -avz run.sh mbr-verify:/opt/verification/run.sh

echo "Restart service"
ssh mbr-verify < restart_verify_service.sh