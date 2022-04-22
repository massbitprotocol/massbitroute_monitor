#!/bin/bash

thread=20
connection=20
duration=20s
rate=10
#dapiURL="http://34.101.82.118"
dapiURL="http://34.101.81.225:8545"

token=SJs5XDqPiU5MPx3h_C2qrA
host=a0f7d53f-b5ff-4ab5-8c5e-a239d81bdaa1.node.mbr.massbitroute.dev

../../scripts/benchmark/wrk -t$thread -c$connection -d$duration -R$rate -s ../../scripts/benchmark/massbit.lua $dapiURL -- $token $host

#response=$(../../scripts/benchmark/wrk -t$thread -c$connection -d$duration -R$rate -s ../../scripts/benchmark/massbit.lua $dapiURL)
#echo $response