#!/bin/bash
dir=/massbit/massbitroute/app/src/sites/services/monitor
cd $dir
_install() {
	#	curl -kL -C - https://download.checkmk.com/checkmk/2.0.0p17/check-mk-raw-2.0.0p17_0.focal_amd64.deb -o check-mk-raw-2.0.0p17_0.focal_amd64.deb
	cat scripts/checkmk/checkmka* >/tmp/check-mk-raw-2.0.0p17_0.focal_amd64.deb
	apt-get update
	dpkg -i /tmp/check-mk-raw-2.0.0p17_0.focal_amd64.deb
	apt-get -f install -y
	sed 's/Listen 80/Listen 8000/' -i /etc/apache2/ports.conf
	rsync -avz etc/check_mk/* /opt/omd/versions/2.0.0p17.cre/lib/python3/cmk/
}

_init() {
	_install
}
$@
