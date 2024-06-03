## Configuration

- Cosmos SDK version

**TARGET_SDK_VERSION**. In order to configure factory you need to provide cosmos sdk version of the target network. You can obtain by performing following command on the network current binary:

```
$ gaiad version --long | grep "cosmos_sdk_version"

cosmos_sdk_version: v0.47.10
```

or using REST service

```
curl -s -X 'GET' \
  'https://api-rs.cosmos.nodestake.top/cosmos/base/tendermint/v1beta1/node_info' \
  -H 'accept: application/json' | jq -r '.application_version.cosmos_sdk_version'

```
- Interchain account registration fee

**IBC_REGISTER_FEE**. Can be queried from `interhaintxs` parameters, using latest binary:

```
$ neutrond query interchaintxs params --node https://rpc-falcron.pion-1.ntrn.tech:443 -o json | jq '.params.register_fee'

[
  {
    "denom": "untrn",
    "amount": "100000"
  }
]
```

or using REST service

```
$ curl -s -X 'GET' \
  'https://rest-falcron.pion-1.ntrn.tech/neutron/interchaintxs/params' \
  -H 'accept: application/json' | jq '.params.register_fee'

[
  {
    "denom": "untrn",
    "amount": "1000000"
  }
]
```

## Running

Export required variables and run script `export $(grep -v '^#' .env.upload.example | xargs) && ./upload_contracts.bash`

