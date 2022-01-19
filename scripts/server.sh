#!/bin/bash
# type=monitor
# dir=/massbit/massbitroute/app/src/sites/services/$type/etc/mkagent/agents
dir=$(dirname $(realpath $0))
pip="pip install"
cd $dir
if [ ! -f "/usr/bin/python3" ]; then
	apt install -y python3
fi

if [ ! -f "/usr/bin/pip" ]; then
	apt install -y python3-pip
fi

export TOKEN_FILE=$dir/tokens.txt
if [ ! -f "$TOKEN_FILE" ]; then touch $TOKEN_FILE; fi

_add() {
	TYPE=$1
	ID=$2
	host=${TYPE}-${ID}
	if [ -z "$host" ]; then
		exit 0
	fi
	export TOKEN=$(echo -n ${host} | sha1sum | cut -d' ' -f1)
	if [ ! -f "$TOKEN_FILE" ]; then touch $TOKEN_FILE; fi
	grep $TOKEN $TOKEN_FILE
	if [ $? -ne 0 ]; then
		echo $TOKEN ${host} >>$TOKEN_FILE
	fi

}
_kill() {
	kill $(ps -ef | grep -v grep | grep -i server.py | awk '{print $2}')
}
if [ $# -eq 0 ]; then
	$pip --upgrade pip
	$pip -r requirements.txt
	python3 $dir/server.py
else
	$@
fi
