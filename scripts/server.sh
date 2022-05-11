#!/bin/bash
# type=monitor
# dir=/massbit/massbitroute/app/src/sites/services/$type/etc/mkagent/agents
dir=$(dirname $(realpath $0))
name=$1
cd $dir

export TOKEN_FILE=$dir/tokens${name}.txt
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
# _kill() {
# 	pkill -f server${name}.py
# }
# if [ $# -eq 0 ]; then
# $pip --upgrade pip
# $pip -r requirements.txt
python3 $dir/server${name}.py
# else
# 	$@
# fi
