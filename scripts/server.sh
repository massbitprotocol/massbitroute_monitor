#!/bin/bash
# type=monitor
# dir=/massbit/massbitroute/app/src/sites/services/$type/etc/mkagent/agents
dir=$(dirname $(realpath $0))
cd $dir

export TOKEN_FILE=$dir/tokens.txt
if [ ! -f "$TOKEN_FILE" ]; then touch $TOKEN_FILE; fi

# _add() {
# 	TYPE=$1
# 	ID=$2
# 	host=${TYPE}-${ID}
# 	if [ -z "$host" ]; then
# 		exit 0
# 	fi
# 	export TOKEN=$(echo -n ${host} | sha1sum | cut -d' ' -f1)
# 	if [ ! -f "$TOKEN_FILE" ]; then touch $TOKEN_FILE; fi
# 	grep $TOKEN $TOKEN_FILE
# 	if [ $? -ne 0 ]; then
# 		echo $TOKEN ${host} >>$TOKEN_FILE
# 	fi

# }
list="$dir/server.py"
# list="$dir/server.py $dir/server_node_eth_mainnet.py $dir/server_gw_eth_mainnet.py"
_kill() {
	for f in $list; do
		pkill -f $f
	done
}
# if [ $# -eq 0 ]; then
# $pip --upgrade pip
# $pip -r requirements.txt
if [ "$1" == "_kill" ]; then
	_kill
	exit 0
fi

# for f in $list; do
python3 $list
# done | parallel -j3
