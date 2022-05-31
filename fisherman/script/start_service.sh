#!/bin/bash

if [ -z "$1" ]
then
    ZONES=( 'as' 'eu' 'na' 'sa' 'oc' 'af' )
else
    ZONES=( "$1" )
fi


for ZN in "${ZONES[@]}"
do
  echo "Stop fisherman at $ZN"
  ssh "mbr-verify-$ZN" 'sudo supervisorctl start fisherman'
done
