#!/bin/bash
apt update
apt install -y python-is-python2 supervisor
mkdir -p /massbit/massbitroute/app/
git clone http://mbr_gateway:6a796299bb72357770735a79019612af228586e7@git.massbitroute.com/massbitroute/gbc.git /massbit/massbitroute/app/gbc

ln -sf /massbit/massbitroute/app/gbc/bin/openresty /usr/local/

git clone http://mbr_gateway:6a796299bb72357770735a79019612af228586e7@git.massbitroute.com/massbitroute/ssl.git /etc/letsencrypt

git clone http://mbr_gateway:6a796299bb72357770735a79019612af228586e7@git.massbitroute.com/massbitroute/asdf.git /massbit/massbitroute/app/gbc/bin/.asdf
cd $(dirname $(realpath $0))
ln -sf /massbit/massbitroute/app/gbc
ln -sf gbc/bin
mkdir tmp db logs
ln -sf gbc/start_server
ln -sf gbc/stop_server
ln -sf gbc/cmd_server
cp supervisor.conf /etc/supervisor/conf.d/mbr_stat.conf
supervisorctl update
