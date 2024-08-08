# Deployment scripts

This directory contains several scripts to assist in deployment of Drop protocol, `upload.bash` and `instantiate.bash`
are those you will need to run.

| Script                    | Purpose                                                                                                             |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| **upload_contracts.bash** | Stores wasm code of all Drop contracts on Neutron chain                                                             |
| **instantiate.bash**      | Creates instance of Drop protocol, waits until ICA addresses are registered and sets them in protocol configuration |
| utils.bash                | Universal library used across deployment scripts. You don't need to execute it by yourself                          |
| migrate.bash              | Simple script which migrates a single contract. Useful for development                                              |

## upload_contracts.bash

### Prerequisities

Before running upload, it is crucial to execute `make build` in root directory of the project. This action ensures
uploaded contracts will correspond to commit hash you are currently on.

### Configuration

Copy `.env.upload.example` to `.env.upload`, then configure it according to this table:

| Parameter           | Suggested testnet value                    | Suggested mainnet value                   | Description                                                                                           |
| ------------------- | ------------------------------------------ | ----------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `NEUTRON_RPC`       | `https://rpc-falcron.pion-1.ntrn.tech:443` | `https://rpc.novel.remedy.tm.p2p.org:443` | Neutron public RPCs taken from chain registry                                                         |
| `GAS_PRICES`        | `0.02`                                     | `0.01`                                    | In case if deployment is too slow and fails on tx timeout, try increasing this value                  |
| `NEUTRON_CHAIN_ID`  | `pion-1`                                   | `neutron-1`                               |                                                                                                       |
| `NEUTRON_HOME`      |                                            |                                           | Set it in case if your Neutron home path is different from default one                                |
| `KEYRING_BACKEND`   |                                            |                                           | Set it to `test`, `os` or whatever backend is in use                                                  |
| `DEPLOY_WALLET`     |                                            |                                           | Set it to name of the wallet you would like to deploy protocol from and then use it as protocol admin |
| `MIN_NTRN_REQUIRED` | `10`                                       | `10`                                      | Scripts check if you have enough funds before doing anything. Generally, better not touch this value  |
| `ARTIFACTS_DIR`     | `../../artifacts`                          | `../../artifacts`                         | Only change it in case if you have moved somewhere either scripts dir or contracts dir                |

### Execution

```bash
export $(grep -v '^#' .env.upload | xargs) && ./upload_contracts.bash
```

After script is finished, please save its output, you will need it for `instantiate.bash`.

## instantiate.bash

### Configuration

Copy `.env.instantiate.example` to `.env.instantiate`, then configure it according to this 2 tables:

#### Client parameters

| Parameter           | Suggested testnet value                    | Suggested mainnet value                   | Description                                                                                           |
| ------------------- | ------------------------------------------ | ----------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `NEUTRON_RPC`       | `https://rpc-falcron.pion-1.ntrn.tech:443` | `https://rpc.novel.remedy.tm.p2p.org:443` | Neutron public RPCs taken from chain registry                                                         |
| `GAS_PRICES`        | `0.02`                                     | `0.01`                                    | In case if deployment is too slow and fails on tx timeout, try increasing this value                  |
| `NEUTRON_CHAIN_ID`  | `pion-1`                                   | `neutron-1`                               |                                                                                                       |
| `NEUTRON_HOME`      |                                            |                                           | Set it in case if your Neutron home path is different from default one                                |
| `TARGET_CHAIN_ID`   |                                            |                                           | Chain ID of target network, could be obtained from chain registry                                     |
| `KEYRING_BACKEND`   |                                            |                                           | Set it to `test`, `os` or whatever backend is in use                                                  |
| `DEPLOY_WALLET`     |                                            |                                           | Set it to name of the wallet you would like to deploy protocol from and then use it as protocol admin |
| `MIN_NTRN_REQUIRED` | `10`                                       | `10`                                      | Scripts check if you have enough funds before doing anything. Generally, better not touch this value  |
| `*_code_id`         |                                            |                                           | Set it to code ID taken from output of upload.bash                                                    |

#### Core parameters

| Parameter                             | Suggested testnet value | Suggested mainnet value | Description                                                                                                                   |
| ------------------------------------- | ----------------------- | ----------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `INITIAL_VALIDATORS`                  |                         |                         | Set it to validators to stake to, format is as follows: `[{"valoper_address":"cosmosvaloper1...","weight":"10"},...]`         |
| `TARGET_BASE_DENOM`                   |                         |                         | Denom to be staked with Drop protocol, e.g. "uatom"                                                                           |
| `TARGET_SDK_VERSION`                  |                         |                         | Cosmos SDK version of target chain, could be obtained from chain registry or from chain itself (refer to documentation below) |
| `NEUTRON_SIDE_TRANSFER_CHANNEL_ID`    |                         |                         | Neutron side channel ID associated with transfer port which is used for IBC transfer of target denom to and from Neutron      |
| `UNBONDING_PERIOD`                    |                         |                         | Can be queried from target chain using `targetd query staking params`, it will be returned as `unbonding_time`                |
| `UNBONDING_SAFE_PERIOD`               | `3600`                  | `3600`                  | Time period before unbonding ends during which we don't initiate any operations just to be safe. One hour is a good default   |
| `UNBOND_BATCH_SWITCH_TIME`            |                         |                         | Divide `UNBONDING_PERIOD` by 7                                                                                                |
| `TIMEOUT_LOCAL`                       | `1209600`               | `1209600`               | 14 days is a good default                                                                                                     |
| `TIMEOUT_REMOTE`                      | `1209600`               | `1209600`               | 14 days is a good default                                                                                                     |
| `STAKER_TIMEOUT`                      | `1209600`               | `1209600`               | 14 days is a good default                                                                                                     |
| `NEUTRON_SIDE_PORT_ID`                | transfer                | transfer                | Neutron side port id to transfer funds                                                                                        |
| `ICQ_UPDATE_PERIOD`                   | 100                     | 100                     | In general we don't need this value since we're using coordinator script for such a purposes. But we need it in core config   |
| `SALT`                                | salt                    | salt                    | `salt` argument in instantiate2 function from cosmwasm_std                                                                    |
| `SUBDENOM`                            |                         |                         | Name of token in factory/.../subdenom                                                                                         |
| `TOKEN_METADATA_DESCRIPTION`          |                         |                         | Token description                                                                                                             |
| `TOKEN_METADATA_DISPLAY`              |                         |                         | Indicates suggested denom that should be displayed in clients                                                                 |
| `TOKEN_METADATA_EXPONENT`             | 6                       | 6                       | Token's exponent. In the best way this parameter shouldn't be changed from suggested value                                    |
| `TOKEN_METADATA_NAME`                 |                         |                         | Token's name                                                                                                                  |
| `TOKEN_METADATA_SYMBOL`               |                         |                         | Symbol is a ticker that usually shown on exchanges. It can be the same as display                                             |
| `CORE_PARAMS_IDLE_MIN_INTERVAL`       | 3600                    | 3600                    | Min interval between state machine changes                                                                                    |
| `CORE_PARAMS_LSM_REDEEM_THRESHOLD`    |                         |                         | Min amount of LSM shares to turn back into staking                                                                            |
| `CORE_PARAMS_LSM_MIN_BOND_AMOUNT`     |                         |                         | Min amount of LSM shares that you can attach as funds in _bond_ call                                                          |
| `CORE_PARAMS_LSM_REDEEM_MAX_INTERVAL` |                         |                         | Interval between 2 LSM redeems (LSM redeem is when we turn LSM share back into staking)                                       |
| `CORE_PARAMS_BOND_LIMIT`              |                         |                         | Max amount of LSM shares that you can attach as funds in _bond_ call                                                          |
| `CORE_PARAMS_MIN_STAKE_AMOUNT`        |                         |                         | Min amount of tokens to transfer to staker contract                                                                           |
| `CORE_PARAMS_ICQ_UPDATE_DELAY`        | 5                       | 5                       |                                                                                                                               |
| `STAKER_PARAMS_MIN_STAKE_AMOUNT`      |                         |                         | Min amount of tokens that staker contract can stake from ICA                                                                  |
| `STAKER_PARAMS_MIN_IBC_TRANSFER`      |                         |                         | Min amount of tokens that staker contract can transfer via IBC to ICA                                                         |

#### Cosmos SDK version

**TARGET_SDK_VERSION**. In order to configure factory you need to provide cosmos sdk version of the target network. You
can obtain by performing following command on the network current binary:

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

### Execution

```bash
export $(grep -v '^#' .env.instantiate | xargs) && ./instantiate.bash
```

During execution the script will print hermes commands. In case if it gets stuck, you will have to execute these hermes
commands manually until ICA account is registered.

After script is finished, write down its output, you will need it to configure frontend, monitoring and hermes.
