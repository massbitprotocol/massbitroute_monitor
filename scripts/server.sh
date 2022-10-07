#!/bin/bash

if [ "$1" == "_kill" ]; then
	pkill -f server.py
	exit 0
fi

SITE_ROOT=$1

export TOKEN_FILE=/tmp/tokens.txt
if [ ! -f "$TOKEN_FILE" ]; then touch $TOKEN_FILE; fi

list="$SITE_ROOT/scripts/server.py"
python3 $list
