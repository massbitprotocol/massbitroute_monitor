## Run check component
```bash
cd target/release/
RUST_LOG=debug RUST_LOG_TYPE=file ./mbr-check-component check-kind -n https://dapi.massbit.io/deploy/info/node/listid -g https://dapi.massbit.io/deploy/info/gateway/listid -c check-flow.json -b base-endpoint.json -o output.json
```
