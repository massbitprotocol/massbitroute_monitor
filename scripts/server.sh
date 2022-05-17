#!/bin/bash
_kill() {
	for f in $list; do
		pkill -f $f
	done
}
if [ "$1" == "_kill" ]; then
	_kill
	exit 0
fi

SITE_ROOT=$1

export TOKEN_FILE=$SITE_ROOT/data/tokens.txt
if [ ! -f "$TOKEN_FILE" ]; then touch $TOKEN_FILE; fi

list="$SITE_ROOT/scripts/server.py"
python3 $list
