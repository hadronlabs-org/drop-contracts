## Activity Generation Script

This script introducing random protocol's execution calls

To prepare script, run:
`yarn install`

To run script, run;
`yarn run start`

To manage script config, open _.env_ file

- CORE_CONTRACT is a protocol instance address of core contract
- MNEMONIC is a wallet mnemonic used to execute core contract's methods
- BASE_DENOM = is an ibc denom used in given protocol instance for bond requests
- FACTORY_DENOM = is an factory denom used in given protocol instance for unbond requests
- NEUTRON_NODE_ADDRESS = endpoint of node o interact with. Should be taken from [here](https://github.com/cosmos/testnets/tree/master/interchain-security/pion-1#endpoints)
- TARGET_NODE_ADDRESS = Node RPC endpoint to sign & broadcast transactions. We need it to do delegate, tokenizeShare and ibc send executions
- BOND_PROB = probabition 'bond' method will be called on core contract
- UNBOND_PROB = probabition 'unbond' method will be called on core contract
- WITHDRAW_PROB = probabition 'send_nft' method will be called on withdrawal_voucher contract
- PROCESS_LSM_PROB = probabition 'processLSMShares' will be executed
- MAX_BOND = Maximum BASE_DENOM amount that we can bond. If current balance lower then MAX_BOND, then MAX_BOND is current BASE_DENOM balance
- MAX_UNBOND = Maximum FACTORY_DENOM amount that we can unbond. If current balance lower then MAX_UNBOND, then MAX_unBOND is current FACTORY_DENOM balance
- MAX_LSM_PROCESS = Maximum BASE_DENOM amount that we can process as lsm shares. If current balance lower then MAX_LSM_PROCESS, then MAX_LSM_PROCESS is current BASE_DENOM balance
- TARGET_CHAIN_PREFIX = Prefix in remote chain account address. For example in case of cosmoshub it's cosmos, in case of osmosis it's osmo etc...
- TARGET_DENOM = Base denom on remote chain
- IBC_CHANNEL_TO = IBC channel to send from neutron to remote chain
- IBC_CHANNEL_FROM = IBC channel to send from remote chain to neutron
- NEUTRON_GASPRICE = Gas price on Neutron chain
- Target_GASPRICE = Gas price on target chain
