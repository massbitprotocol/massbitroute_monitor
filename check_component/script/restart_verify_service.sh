sudo mv ~/mbr-check-component /opt/verification/
sudo mv ~/check-flow.json /opt/verification/
sudo mv ~/base-endpoint.json /opt/verification/
sudo mv ~/config_check_component.json /opt/verification/

sudo supervisorctl restart verification
