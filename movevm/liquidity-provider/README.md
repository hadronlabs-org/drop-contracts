# Initia MoveVM "provide liquidity and send LP" module

#### 1. Prepare an empty address with INIT tokens

You might need to use a [faucet](https://faucet.testnet.initia.xyz/) for that.
This way, you should have:

- initiad binary;
- mnemonic with some INIT tokens on initiation-2 network.

#### 2. Configure deployment

Navigate to `Move.toml` and open it in your editor of choice. You are interested in the
section `[addresses]`. `me`, `backup_owner` and `recipient` are filled with placeholder (`_`) addresses,
so you will have to fill them up. You can use this easy snippet to generate hexadecimal
addresses from keys stored in your initiad keychain:

```bash
NAME="<name of your key>"; echo "0x$(initiad keys parse "$(initiad keys show "$NAME" --output json | jq -r '.address')" --output json | jq -r '.bytes' | tr '[:upper:]' '[:lower:]')"
```

First, fill `me` to the address of your own account. For `recipient`, you can create a new
empty account and use it's address.

#### 3. Build module

It is as easy as `initiad move build`.

#### 4. Deploy module

It is also easy, sign your normal Cosmos SDK transaction:

```bash
initiad move deploy --path "$(pwd)" --upgrade-policy COMPATIBLE --from <name of your key> --gas auto --gas-adjustment 1.5 --gas-prices 0.025uinit --node https://rpc.initiation-2.initia.xyz:443 --chain-id initiation-2
```

#### 5. Determine address of module object

Open module upload transaction in block explorer, for example take a look at
[this one](https://scan.testnet.initia.xyz/initiation-1/txs/7B408B00337E840D0AF2BB89615CEEFFD73A28458D1BD31185418909FAF37BDB).
Look for event log with `type_tag` equal to `0x1::object::CreateEvent`.
Inside this event there is a JSON, containing a field `object` with the
address of our new module object. This address is where INIT tokens are expected
to be deposited to.

#### 6. Send INIT tokens to the module object

Should be as easy as a normal Cosmos SDK bank transfer, with a single caveat.
You have object address in HEX format, but you need bech32. Let's convert it:
suppose you have address `0x8a6fc188562db0e6008896b4e7a5ec027fe3461cb4169adc5165d1b58732d720`.
Run `initiad keys parse 8a6fc188562db0e6008896b4e7a5ec027fe3461cb4169adc5165d1b58732d720`,
take the first output with `init1` prefix: `init13fhurzzk9kcwvqygj66w0f0vqfl7x3sukstf4hz3vhgmtpej6usqzqa0mq`,
this would be the address to send funds to:

```bash
initiad tx bank send <name of your key> init13fhurzzk9kcwvqygj66w0f0vqfl7x3sukstf4hz3vhgmtpej6usqzqa0mq 4242uinit --gas auto --gas-adjustment 1.5 --gas-prices 0.025uinit --chain-id initiation-2 --node https://rpc.initiation-2.initia.xyz:443
```

#### 7. Execute contract

```bash
initiad tx move execute <address of @me from Move.toml> liquidity_provider provide --from testnet --gas auto --gas-adjustment 1.5 --gas-prices 0.025uinit --node https://rpc.initiation-2.initia.xyz:443 --chain-id initiation-2
```

#### 8. Validate

Use block explorer to validate that:

- @me address doesn't have any INIT tokens anymore;
- @recipient address has some LP tokens (denom is
  `move/dbf06c48af3984ec6d9ae8a9aa7dbb0bb1e784aa9b8c4a5681af660cf8558d7d`).
