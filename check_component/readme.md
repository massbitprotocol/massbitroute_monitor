## Run check component
```bash
cd target/release/
RUST_LOG=debug RUST_LOG_TYPE=file ./mbr-check-component check-kind -n 'https://dapi.massbit.io/deploy/info/node/listid' -g 'https://dapi.massbit.io/deploy/info/gateway/listid' -d 'https://dapi.massbit.io/deploy/info/dapi/listid' -c check-flow.json -b base-endpoint.json -o output.json
```
## Query status of a gateway
```bash
curl -X POST \
http://0.0.0.0:3030/get_status \
-H 'cache-control: no-cache' \
-H 'content-type: application/json' \
-H 'postman-token: d83bf53d-4413-47d3-df34-1567680bcc6d' \
-d '{
"blockchain": "eth",
"network": "mainnet",
"id": "3bfd9189-3b44-4de1-9e32-de98be718543",
"user_id": "298eef2b-5fa2-4a3d-b00c-fe95b01e237c",
"ip": "34.150.13.159",
"zone": "",
"country_code": "",
"token": "",
"component_type": "Gateway"
}'
```

# Start script for ethereum node
```bash
#!/bin/bash
add-apt-repository ppa:ethereum/ethereum
apt install -y ethereum
# setup new user
ETH_HOME=/home/ethereum/
ETH_USER=ethereum
SERVICE=/etc/systemd/system/ethereum.service
RUN_SCRIPT=/nodes/ethereum/run.sh
mkdir -p /nodes/ethereum
chown ${ETH_USER}:${ETH_USER} -R /nodes/ethereum

sudo mkdir "${ETH_HOME}"
sudo chmod -R 757 "${ETH_HOME}"
sudo chmod -R 757 /nodes/ethereum/
sudo adduser --disabled-password --gecos "" --home "${ETH_HOME}" "${ETH_USER}"
# create systemd
sudo cat >${SERVICE} <<EOL
[Unit]
      Description=Geth Node
      After=network.target
[Service]
      LimitNOFILE=700000
      LogRateLimitIntervalSec=0
      User=ethereum
      Group=ethereum
      WorkingDirectory=/nodes/ethereum/
      Type=simple
      ExecStart=/nodes/ethereum/run.sh
      StandardOutput=file:/home/ethereum/console.log
      StandardError=file:/home/ethereum/error.log
      Restart=always
      RestartSec=10
[Install]
      WantedBy=multi-user.target
EOL
sudo cat >${RUN_SCRIPT} <<EOL
#!/usr/bin/bash
/usr/bin/geth --syncmode "swap" --nousb --http --http.addr 0.0.0.0 --http.api db,eth,net,web3,personal,shh --http.vhosts "*" --http.corsdomain "*" --ws --ws.addr 0.0.0.0 --ws.origins "*" --ws.api db,eth,net,web3,personal,shh 2>&1  >> /nodes/ethereum/eth.log
EOL
chmod +x $RUN_SCRIPT
chown ${ETH_USER}:${ETH_USER} -R /nodes/ethereum
systemctl enable ethereum.service
systemctl start ethereum.service
```
