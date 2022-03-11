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

