#!/bin/bash
cargo build --release

ZONES=( 'as' )

for ZN in "${ZONES[@]}"
do
  echo "Update bin file in zone $ZN"
  rsync -avz ../target/release/mbr-stats "mbr-verify-$ZN:~/mbr-stats"
  rsync -avz ../.env "mbr-verify-$ZN:~/.env"

  #Update run.sh later
  cat run.tpl | sed "s/\[\[ZONE\]\]/$ZN/g" > _run_$ZN.sh
  rsync -avz _run_$ZN.sh "mbr-verify-$ZN:~/run.sh"
  rm _run_$ZN.sh

  echo "Restart service"
  ssh "mbr-verify-$ZN" < restart_service.sh
done