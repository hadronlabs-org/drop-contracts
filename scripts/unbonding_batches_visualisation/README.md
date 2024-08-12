## Unbonding Batches Visualisation

This script introducing the batches visualisation

To prepare script, run:
`npm install`

To run script, run;
`npm run start`

To manage script config, open _.env_ file

- MODE can be either FULL or RECENT
  - FULL mode will get all batches from current to 0
  - RECENT mode will get all batches from current to first with status _withdrawn_
- CORE_CONTRACT is an address of protocol's core contract that we're using to query batches
- NODE_ADDRESS node address to interact with given smart contract
- WALLET_MNEMONIC no transactions are executed from this script. We need this field only to init DirectSecp256k1HdWallet, so it doesn't matter what this field contain

if you want to get pretty table of contents in your terminal, use the following command:

```bash
node -r ts-node/register --env-file=./.env src/app.ts | jq -r '(["id","status","expected","creation(UTC)","finalization(UTC)","unstaked"] | (., map(length*"-"))), (.[] | [.batch_id,.status,.expected_amount // "-",.creation_time,.expected_finalization_time // "-",.unstaked_amount // "-"]) | @tsv' --raw-output | column -t
```
