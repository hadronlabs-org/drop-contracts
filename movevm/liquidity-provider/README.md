# Initia MoveVM "provide liquidity and send LP" module

#### 1. Prepare an empty address with INIT tokens

You might need to use a [faucet](https://faucet.testnet.initia.xyz/) for that.
This way, you should have:

- initiad binary;
- mnemonic with some INIT tokens on initiation-2 network.

#### 2. Get your account address in hexademical format

```bash
NAME="<name of your key>"; echo "0x$(initiad keys parse "$(initiad keys show "$NAME" --output json | jq -r '.address')" --output json | jq -r '.bytes' | tr '[:upper:]' '[:lower:]')"
```

#### 3. Build module

It is as easy as `initiad move build --named-addresses "me=<your hex address from step 2>"`.

#### 4. Deploy module

It is also easy, sign your normal Cosmos SDK transaction:

```bash
initiad move deploy --path "$(pwd)" --upgrade-policy COMPATIBLE --from <name of your key> --gas auto --gas-adjustment 1.5 --gas-prices 0.025uinit --node https://rpc.initiation-2.initia.xyz:443 --chain-id initiation-2
```

#### 5. Instantiate a new liquidity provider object

Method `create_liquidity_provider` has several arguments:

- string:\<name\> Name for a visual reference, e.g. "testnet_uinit"
- option\<address\>:\<address\> Priviliged account with a right to withdraw any coins from module object, will be set to `account` if omitted
- string:\<name\> Name of a slinky pair, e.g. "INIT/USD"
- object:\<address\> Address of liquidity pool, e.g. 0xdbf06c48af3984ec6d9ae8a9aa7dbb0bb1e784aa9b8c4a5681af660cf8558d7d for uinit-usdc on initiation-2 testnet
- object:\<address\> Address of asset, e.g. 0x8e4733bdabcf7d4afc3d14f0dd46c9bf52fb0fce9e4b996c939e195b8bc891d9 for uinit on initiation-2 testnet
- address:\<address\> Address which will be receiving all LP tokens

```bash
initiad tx move execute <your hex address from step 2> drop_lp create_liquidity_provider --args '["string:<name>", "option<address>:null", "string:INIT/USD", "object:<lp_metadata_address>", "object:<input_token_address>", "address:<lp_recepient>"]' --from test2 --node $INITIA_TESTNET --chain-id initiation-2 --gas auto --gas-adjustment 1.5 --gas-prices 0.025uinit
```

Example:

```bash
initiad tx move execute 0x8b4ab83f91eef29b3d0211d7a9332ba44c818a5c drop_lp create_liquidity_provider --args '["string:test_uinit", "option<address>:null", "string:INIT/USD", "object:0xdbf06c48af3984ec6d9ae8a9aa7dbb0bb1e784aa9b8c4a5681af660cf8558d7d", "object:0x8e4733bdabcf7d4afc3d14f0dd46c9bf52fb0fce9e4b996c939e195b8bc891d9", "address:0x8b4ab83f91eef29b3d0211d7a9332ba44c818a5c"]' --from test2 --node $INITIA_TESTNET --chain-id initiation-2 --gas auto --gas-adjustment 1.5 --gas-prices 0.025uinit
```

#### 6. Provide liquidity

First, go to the initia [faucet](https://faucet.testnet.initia.xyz/),
then get some tokens on the liquidity_provider instance address. But you need sdk-type address,
you can pick it up from the events from the transaction in step 5. Then convert this address
into sdk-type with init1... prefix (you can get one on inita scan because initiad doesn't work properly with long addresses).

- address:\<address\> Address of a liquidity provider instance

```bash
initiad tx move execute <name of your key> drop_lp provide '["address:<hex_lp_address>"]' --from testnet --gas auto --gas-adjustment 1.5 --gas-prices 0.025uinit --node https://rpc.initiation-2.initia.xyz:443 --chain-id initiation-2
```

#### 8. Validate

Use block explorer to validate that:

- lp_provider address doesn't have any INIT tokens anymore;
- @recipient address has some LP tokens (denom is
  `move/dbf06c48af3984ec6d9ae8a9aa7dbb0bb1e784aa9b8c4a5681af660cf8558d7d`).
