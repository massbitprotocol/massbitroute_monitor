#!/bin/bash
#ZONES=( 'as' 'eu' 'na' 'sa' 'af' 'oc')
ZONES=( 'sa' )
for ZN in "${ZONES[@]}"
do
  echo "Service status at $ZN"
  ssh "mbr-verify-$ZN" 'sudo supervisorctl status all'
done