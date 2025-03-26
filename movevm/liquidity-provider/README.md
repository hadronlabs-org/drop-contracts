# Initia MoveVM "Liquidity Provider" module

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

It is as easy as

```bash
initiad move build --named-addresses "me=<your hex address from step 2>"
```

#### 4. Deploy module

It is also as easy as

```bash
initiad move deploy --path "$(pwd)" --upgrade-policy COMPATIBLE --from <name of your key> --gas auto --gas-adjustment 1.5 --gas-prices 0.025uinit --node https://rpc.initiation-2.initia.xyz:443 --chain-id initiation-2
```

#### 5. Instantiate a new Liquidity Provider object

Method `create_liquidity_provider` has several arguments:

- string:\<name\> Name for a visual reference, e.g. "testnet_uinit"
- address:\<address\> Priviliged account with a permission to withdraw any coins from the module object, will be set to `account` if omitted
- string:\<name\> Name of a slinky pair, e.g. "INIT/USD"
- object:\<address\> Address of liquidity pool, e.g. 0xdbf06c48af3984ec6d9ae8a9aa7dbb0bb1e784aa9b8c4a5681af660cf8558d7d for uinit-usdc on initiation-2 testnet
- object:\<address\> Address of asset, e.g. 0x8e4733bdabcf7d4afc3d14f0dd46c9bf52fb0fce9e4b996c939e195b8bc891d9 for uinit on initiation-2 testnet
- address:\<address\> Address which will be receiving all LP tokens (Rewards Pump ICA on Initia)

```bash
initiad tx move execute <your hex address from the step 2> drop_lp create_liquidity_provider --args '["string:<name>", "address:<backup address>", "string:INIT/USD", "object:<lp_metadata_address>", "object:<input_token_address>", "address:<lp_recepient>"]' --from <name of your key> --node https://rpc.initiation-2.initia.xyz:443 --chain-id initiation-2 --gas auto --gas-adjustment 1.5 --gas-prices 0.025uinit
```

To get Liquidity Provider instance object's address use this:

```bash
initiad q tx <tx hash from the previous transaction> --node https://rpc.initiation-2.initia.xyz:443 -o j | jq '.events[] | select(.attributes[].value | contains("CreateLiquidityProviderEvent")) | .attributes[] | select(.key == "data").value | fromjson.lp_address' | sed 's/\"//g'
```

Then, to convert it from hex to bech32, use this command:

```bash
initiad keys parse <hex address without 0x prefix>
```

And pick up the one with "init1..." prefix.

#### 6. Provide liquidity

First, go to the initia [faucet](https://faucet.testnet.initia.xyz/),
then get some INIT tokens on the Liquidity Provider bech32 instance address that you got from the step 5.

- address:\<address\> Hex address of the liquidity provider instance

```bash
initiad tx move execute <your hex address from step 2> drop_lp provide '["address:<hex_lp_address>"]' --from <name of your key> --gas auto --gas-adjustment 1.5 --gas-prices 0.025uinit --node https://rpc.initiation-2.initia.xyz:443 --chain-id initiation-2
```

#### 8. Validate

Use block explorer to validate that:

- Liquidity Provider address doesn't have any INIT tokens anymore;
- @recipient address has some LP tokens (denom is
  `move/dbf06c48af3984ec6d9ae8a9aa7dbb0bb1e784aa9b8c4a5681af660cf8558d7d`).
