sudo mv ~/mbr-check-component /opt/verification/
sudo mv ~/check-flow.json /opt/verification/
sudo mv ~/base-endpoint.json /opt/verification/
sudo mv ~/config_check_component.json /opt/verification/
sudo mv ~/run.sh /opt/verification/
sudo mv ~/massbit.lua /opt/verification/
sudo mv ~/wrk /opt/verification/

sudo supervisorctl restart verification
