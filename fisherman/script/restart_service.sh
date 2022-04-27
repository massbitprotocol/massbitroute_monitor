sudo mv ~/mbr-fisherman /opt/fisherman/
sudo mv ~/check-flow.json /opt/fisherman/
sudo mv ~/base-endpoint.json /opt/fisherman/
sudo mv ~/config_check_component.json /opt/fisherman/
sudo mv ~/config_fisherman.json /opt/fisherman/
sudo mv ~/massbit.lua /opt/fisherman/
sudo mv ~/wrk /opt/fisherman/
sudo mv ~/.env /opt/fisherman/

sudo supervisorctl restart fisherman
