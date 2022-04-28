#!/bin/bash
cargo build --release
#ZONES=( 'as' 'eu' 'na' 'sa' 'af' 'oc' )
ZONES=( 'as')

for ZN in "${ZONES[@]}"
do
  echo "Update bin file in zone $ZN"
  rsync -avz ../target/release/mbr-check-component "mbr-verify-$ZN:~/mbr-check-component"
  rsync -avz ../src/archive/check-flow.json "mbr-verify-$ZN:~/check-flow.json"
  rsync -avz ../src/archive/base-endpoint.json "mbr-verify-$ZN:~/base-endpoint.json"
  rsync -avz ../config_check_component.json "mbr-verify-$ZN:~/config_check_component.json"
  rsync -avz ../../scripts/benchmark/massbit.lua "mbr-verify-$ZN:~/massbit.lua"
  rsync -avz ../../scripts/benchmark/wrk "mbr-verify-$ZN:~/wrk"
  rsync -avz ../.env "mbr-verify-$ZN:~/.env"

  cat run.tpl | sed "s/\[\[ZONE\]\]/$ZN/g" > _run_$ZN.sh
  rsync -avz _run_$ZN.sh "mbr-verify-$ZN:~/run.sh"
  rm _run_$ZN.sh

  echo "Restart service"
  ssh "mbr-verify-$ZN" < restart_verify_service.sh
done