#!/bin/bash

if [ -z "$1" ]
then
    ZONES=( 'as' 'eu' 'na' 'sa' 'oc' 'af' )
else
    ZONES=( "$1" )
fi


for ZN in "${ZONES[@]}"
do
  echo "Service status at $ZN"
  ssh "mbr-verify-$ZN" 'sudo supervisorctl status all; sudo cat /opt/verification/run.sh; sudo cat /opt/fisherman/run.sh; sudo cat /opt/stats/run.sh'
done
