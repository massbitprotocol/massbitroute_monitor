#!/bin/bash
ZONES=( 'as' 'eu' 'na' 'sa' 'af' 'oc')

for ZN in "${ZONES[@]}"
do
  echo "Stop fisherman at $ZN"
  ssh "mbr-verify-$ZN" 'sudo supervisorctl stop fisherman'
done
