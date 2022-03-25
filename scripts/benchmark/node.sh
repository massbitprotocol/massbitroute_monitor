#!/bin/bash
thread=100
connection=100
duration=160s
rate=10000
id=$1
token=$2

id=$1
token=$2
if [ $# -ne 2 ]; then
	echo "$0 ID TOKEN"
	exit 1
fi
./wrk -t$thread -c$connection -d$duration -R$rate -s massbit.lua https://${id}.node.mbr.massbitroute.com/ -- $token
exit 0
