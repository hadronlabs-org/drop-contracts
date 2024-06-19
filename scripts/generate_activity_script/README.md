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
- BOND_PROB = probabition 'bond' method will be called on core contract
- UNBOND_PROB = probabition 'unbond' method will be called on core contract
- WITHDRAW_PROB = probabition 'send_nft' method will be called on withdrawal_voucher contract
