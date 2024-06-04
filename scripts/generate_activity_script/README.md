## Activity Generation Script

This script introducing random protocol's execution calls

To prepare script, run:
`npm install`

To run script, run;
`npm run start`

To manage script config, open _.env_ file

- TARGET is a protocol instance address of core contract
- MNEMONIC is a wallet mnemonic used to execute core contract's methods
- IBC_DENOM = is an ibc denom used in given protocol instance for bond requests
- FACTORY_DENOM = is an factory denom used in given protocol instance for unbond requests
- NODE_ADDRESS = endpoint of node o interact with. Should be taken from [here](https://github.com/cosmos/testnets/tree/master/interchain-security/pion-1#endpoints)
- BOND_PROB = probabition 'bond' method will be called on core contract
- UNBOND_PROB = probabition 'unbond' method will be called on core contract
- WITHDRAW_PROB = probabition 'send_nft' method will be called on withdrawal_voucher contract

# Little caveat

- BOND_PROB + UNBOND_PROB + WITHDRAW_PROB should be equal to 1.

- If the result of random choosing is the method that will be executed with error (it can appear if there is nothing to bond, unbond or send_nft), there will be chosen other method (with equal probabition) except one what was provided in .env file.

- If each possible call (bond, unbond or send_nft) falls with error then process finishing with exit code 1.
