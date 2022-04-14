#!/bin/bash
cargo build --release

ZONES=('as' 'eu' 'na' 'sa' 'af' 'oc')

for ZN in "${ZONES[@]}"
do
  echo "Update bin file in zone $ZN"
  rsync -avz ../target/release/mbr-fisherman "mbr-verify-$ZN:~/mbr-fisherman"
  rsync -avz ../../check_component/src/archive/check-flow.json "mbr-verify-$ZN:~/check-flow.json"
  rsync -avz ../../check_component/src/archive/base-endpoint.json "mbr-verify-$ZN:~/base-endpoint.json"
  rsync -avz ../config_check_component.json "mbr-verify-$ZN:~/config_check_component.json"
  rsync -avz ../config_fisherman.json "mbr-verify-$ZN:~/config_fisherman.json"

  #Update run.sh later

  echo "Restart service"
  ssh "mbr-verify-$ZN" < restart_service.sh
done