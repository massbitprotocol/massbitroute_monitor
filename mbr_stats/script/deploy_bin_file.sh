#!/bin/bash

echo "Update bin file"
rsync -avz ../target/release/mbr-stats mbr-verify:/opt/stats/mbr-stats

echo "Update run script"
rsync -avz run.sh mbr-verify:/opt/stats/run.sh

echo "Restart service"
ssh mbr-verify < restart_service.sh