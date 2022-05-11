#!/bin/bash
cargo build --release

ZONES=( 'as' 'na' 'eu' )

for ZN in "${ZONES[@]}"
do
  echo "Update bin file in zone $ZN"
  rsync -avz ../target/release/mbr-fisherman "mbr-verify-$ZN:~/mbr-fisherman"
  rsync -avz ../../check_component/src/archive/check-flow.json "mbr-verify-$ZN:~/check-flow.json"
  rsync -avz ../../check_component/src/archive/base-endpoint.json "mbr-verify-$ZN:~/base-endpoint.json"
  rsync -avz ../config_check_component.json "mbr-verify-$ZN:~/config_check_component.json"
  rsync -avz ../config_fisherman.json "mbr-verify-$ZN:~/config_fisherman.json"
  rsync -avz ../../scripts/benchmark/massbit.lua "mbr-verify-$ZN:~/massbit.lua"
  rsync -avz ../../scripts/benchmark/wrk "mbr-verify-$ZN:~/wrk"
  rsync -avz ../.env "mbr-verify-$ZN:~/.env"

  #Update run.sh later
  cat run.tpl | sed "s/\[\[ZONE\]\]/$ZN/g" > _run_$ZN.sh
  rsync -avz _run_$ZN.sh "mbr-verify-$ZN:~/run.sh"
  rm _run_$ZN.sh

  echo "Restart service"
  ssh "mbr-verify-$ZN" < restart_service.sh
done