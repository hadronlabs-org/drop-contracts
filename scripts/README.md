## Configuration

In order to configure factory you need to provide cosmos sdk version of the target network. You can obtain by perforoming following command on the network current binary.

```
$ gaiad version --long | grep "cosmos_sdk_version"

cosmos_sdk_version: v0.47.10
```

## Running

Export required variables and run script `export $(grep -v '^#' .env.upload.example | xargs) && ./upload_contracts.bash`

