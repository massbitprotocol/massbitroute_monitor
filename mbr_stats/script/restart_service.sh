sudo mv ~/mbr-stats /opt/stats/
sudo mv ~/run.sh /opt/stats/
sudo mv ~/.env /opt/stats/

sudo supervisorctl restart stats
