#!/bin/bash
dir=/massbit/massbitroute/app/src/sites/services/monitor
cd $dir
_install() {
	curl -kL -C - https://download.checkmk.com/checkmk/2.0.0p17/check-mk-raw-2.0.0p17_0.focal_amd64.deb -o check-mk-raw-2.0.0p17_0.focal_amd64.deb
	dpkg -i check-mk-raw-2.0.0p17_0.focal_amd64.deb
	apt-get -f install -y
	sed 's/Listen 80/Listen 8000/' -i /etc/apache2/ports.conf
	/etc/init.d/apache2 restart
	rsync -avz etc/check_mk/* /opt/omd/versions/2.0.0p17.cre/lib/python3/cmk/
}

_add() {
	_name=$1
	omd create $_name

	# mkdir -p etc/mkagent
	# git clone http://mbr_gateway:6a796299bb72357770735a79019612af228586e7@git.massbitroute.com/massbitroute/mkagent.git  etc/mkagent
	# ln -sf /massbit/massbitroute/app/src/sites/services/monitor/etc/mkagent/agents/main.mk /opt/omd/sites/mbr/etc/check_mk/main.mk
	omd start $_name
	su - $_name -c "htpasswd /opt/omd/sites/$_name/etc/htpasswd cmkadmin"
}
$@
