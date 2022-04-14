#!/bin/bash
cargo build --release
ZONES=( 'as' 'eu' 'na' 'sa' 'af' 'oc' )

for ZN in "${ZONES[@]}"
do
  echo "Update bin file in zone $ZN"
  rsync -avz ../target/release/mbr-check-component "mbr-verify-$ZN:~/mbr-check-component"
  rsync -avz ../src/archive/check-flow.json "mbr-verify-$ZN:~/check-flow.json"
  rsync -avz ../src/archive/base-endpoint.json "mbr-verify-$ZN:~/base-endpoint.json"
  rsync -avz ../config_check_component.json "mbr-verify-$ZN:~/config_check_component.json"

#  echo "Update run script"
#  rsync -avz run.sh "mbr-verify-$ZN:/opt/verification/run.sh"

  echo "Restart service"
  ssh "mbr-verify-$ZN" < restart_verify_service.sh
done