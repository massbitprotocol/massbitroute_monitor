#!/bin/bash
#ZONES=( 'as' 'eu' 'na' 'sa' 'af' 'oc')
ZONES=( 'as' 'eu' 'na' 'sa' 'af' 'oc' )
for ZN in "${ZONES[@]}"
do
  echo "Clean up at $ZN"
  ssh "mbr-verify-$ZN" 'sudo rm /root/.cargo /root/.rustup -rf'
done