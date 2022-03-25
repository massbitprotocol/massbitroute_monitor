#!/bin/bash
thread=100
connection=100
duration=160s
rate=10000
id=$1
token=$2
if [ $# -ne 1 ]; then
	echo "$0 ID"
	exit 1
fi

./wrk -t$thread -c$connection -d$duration -R$rate -s massbit.lua https://${id}.gw.mbr.massbitroute.com
exit 0
