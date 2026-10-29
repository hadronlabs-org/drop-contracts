# Verifying the dATOM recovery proposal

This is for Drop Admin signers. Follow the steps in order. Each one says what you should see. Any mismatch → don't vote yes.

Needed: `neutrond` (v11+), `jq`, `git`, Docker. Section 2 also needs `python3`. Commands use zsh/bash syntax.

```sh
NODE=https://neutron-rpc.polkachu.com:443
PROP_MODULE=neutron1cj68y3e82awra7hsyxz0dl70gaazmzf8pvc9svxszzeafdd9gwlsn9lcyg
DAO=neutron1xm4xgfv4xz4ccv0tjvlfac5gqwjnv9zzx4l47t7ve7j2sn4k7gwqkg947d
FACTORY=neutron1rtydk5vlppj2nmw98ctpgee6hxe5va7mc9x3lng8xm75p3dus80s7pkvmu
WM=neutron1gcklauurx4ckyp0t8a6axl0500vy3n09y5utpspjg8ruxnln8erq2uumck
CONV=neutron15024sa6het7qrgccrx9wtxff7mh9qrcu9m6gvvlvez6em46ylscqk4kqj9
ATOM=ibc/C4CFF46FD6DE35CA4CF4CE031E643C8FDC9BA4B99AE598E9B0ED98FE3A2319F9
DATOM=factory/neutron1k6hr0f83e7un2wjf29cspk7j69jrnskk65k3ek2nj9dztrlzpj6q00rtsa/udatom
PROP_ID=<id>
```

## 1. The proposal contains exactly the expected messages

```sh
neutrond q wasm contract-state smart $PROP_MODULE "{\"proposal\":{\"proposal_id\":$PROP_ID}}" --node $NODE -o json \
  | jq '.data.proposal.msgs
        | map(.wasm.execute | {contract_addr, funds, msg: (.msg | @base64d | fromjson)})
        | (.[0].msg.admin_execute.msgs[0].wasm.migrate.msg |= (@base64d | fromjson))'
```
Expected output, with exactly 2 messages:
- `[0]`: `contract_addr` = `$FACTORY`, `funds` = `[]`, msg = `admin_execute` with **one** `migrate`: `contract_addr` = `$WM`, `new_code_id` = `5412`, msg = `{"payout_ratio":"0.7121"}`.
- `[1]`: `contract_addr` = `$CONV`, `funds` = `[]`, msg = `{"update_rate":{"rate":"0.9266621389"}}`.

There should be no other messages, no bank sends, no `update_admin` and no `update_ownership`.

## 2. Code 5412 is what the source says

### 2a. On-chain hash
```sh
neutrond q wasm code-info 5412 --node $NODE
# data_hash: 04F4E59156C299405755A8D2B5A1DAA2C91F29213719412F18716D1E2AC6420A
# instantiate_permission: AnyOfAddresses [ $FACTORY ]   (see "Before the proposal executes" in PROPOSAL.md)
```
Hash the actual bytes rather than relying on `data_hash`:
```sh
neutrond q wasm code 5412 /tmp/5412.wasm --node $NODE && shasum -a 256 /tmp/5412.wasm
```

### 2b. The source change is small and does what it claims
```sh
git clone https://github.com/hadronlabs-org/drop-contracts && cd drop-contracts
git log --oneline v1.0.0..da87d2cf       # exactly one commit: "add payout ratio"
git diff v1.0.0 da87d2cf --stat          # withdrawal-manager + packages/base withdrawal_manager msg/state only
git diff v1.0.0 da87d2cf
```
Check in the diff:
- `payout_amount = user_share × unbonded_amount × payout_ratio`. When no ratio is stored, the ratio is 1.
- `migrate` stores `payout_ratio` only if it is above 0 and at most 1, and bumps the version to 1.0.1.
- The new `PayoutRatio {}` query is read-only.
- `Config`, `CONFIG` and the ownership code are unchanged.

Tests: `cargo test -p drop-withdrawal-manager` → 5 passed.

### 2c. v1.0.0 is really the source of the currently deployed code 1728
This needs Docker. It runs under emulation on Apple Silicon and takes about 30–60 minutes.
```sh
git checkout v1.0.0
docker run --rm -v "$PWD":/code --mount type=volume,source=v100_cache,target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  --platform linux/amd64 cosmwasm/workspace-optimizer:0.15.1
grep withdrawal_manager artifacts/checksums.txt
# 35f6f4b9e2c2b9326d15f0cfc1a1056c7d4e4b08b02d80daadfd0d473a4e9e5e
neutrond q wasm code-info 1728 --node $NODE   # data_hash 35F6F4B9…9E5E
```

### 2d. Reproduce 5412
Neutron's wasmvm (cosmwasm-vm ≥ 3.0.9) rejects any function with more than 100 locals. The plain `make compile` build of this contract has 113 in `execute`, and that build's checksum is `d950fc76…`. So 5412 is built with `make compile_neutron`, added in commit `6a35c789` and explained in the repo README under "Production builds for Neutron". It uses the same image plus one extra `wasm-opt` flag, which turns off inlining of functions that have a single caller. It does not change contract behaviour. Commit `6a35c789` changes only the Makefile and README, so the contract source is identical to `da87d2cf`.
```sh
git checkout fix/datom-wm-haircut    # contains da87d2cf + 6a35c789
git diff da87d2cf HEAD --stat -- contracts packages   # no output: contract source unchanged
make compile_neutron                 # ends with `sudo chown` on artifacts/
grep withdrawal_manager artifacts/checksums.txt
# 04f4e59156c299405755a8d2b5a1daa2c91f29213719412f18716d1e2ac6420a
```
You must build for `linux/amd64` (the target already sets this), because arm64 builds produce different hashes.

## 3. The numbers

### 3a. State just before the drain (block 61633264)
The archive node must still have that height. polkachu kept it as of 2026-10-29.
```sh
for A in $CONV $WM; do neutrond q bank balances $A --height 61633264 --node $NODE -o json | jq -c .balances; done
neutrond q bank total-supply-of $DATOM --height 61633264 --node $NODE
```
Expected:
| | value |
|---|---|
| converter ATOM | 1,589,711.919394 |
| converter dATOM (already converted) | 735,337.205615 |
| WM ATOM | 115,878.183542 |
| dATOM total supply (same now) | 1,956,403.298557 |

### 3b. Liabilities
- WM: Σ(`unbonded_amount − withdrawn_amount`) over all `withdrawn` batches is **115,878.183541 ATOM**. That equals the pre-drain balance, so the contract was fully funded.
  ```sh
  neutrond q wasm contract-state smart neutron16m3hjh7l04kap086jgwthduma0r5l0wh8kc6kaqk92ge9n5aqvys9q6lxr \
    '{"unbond_batches":{"limit":"500"}}' --node $NODE -o json \
    | jq '[.data.unbond_batches[] | select(.status=="withdrawn") | ((.unbonded_amount|tonumber) - (.withdrawn_amount // "0" | tonumber))] | add / 1e6'
  ```
- Converter: circulating dATOM = supply − dATOM held by the converter = 1,956,403.298557 − 735,337.205615 = **1,221,066.092942 dATOM**. At the old rate 1.301309 that needs 1,588,984 ATOM. The converter held 1,589,712, so it was fully funded.

### 3c. Ratio and solvency
```sh
python3 -c '
wm_owed=115878.183541; circ=1956403.298557-735337.205615
r=0.7121; rate=1.301309*r
print("rate", rate)                                        # 0.9266621389
print("WM needs", wm_owed*r, "refill 82520")               # 82516.85 <= 82520
print("CONV needs", circ*rate, "refill 1132075")           # 1131516  <= 1132075
print("recovery WM", 82520/115878.183542, "CONV", 1132075/1589711.919394)  # 0.712127 / 0.712126
'
```
Both refills cover 71.21% of the liability, with a little to spare. Because the contracts round payouts down, they can't pay out more than they hold.

## 4. Simulate the proposal as the DAO, read-only
This runs each message against live state as if the DAO sent it. Nothing is broadcast.
```sh
neutrond tx wasm execute $FACTORY \
  '{"admin_execute":{"msgs":[{"wasm":{"migrate":{"contract_addr":"'$WM'","new_code_id":5412,"msg":"eyJwYXlvdXRfcmF0aW8iOiIwLjcxMjEifQ=="}}}]}}' \
  --from $DAO --chain-id neutron-1 --node $NODE --gas-prices 0.0055untrn --dry-run
neutrond tx wasm execute $CONV '{"update_rate":{"rate":"0.9266621389"}}' \
  --from $DAO --chain-id neutron-1 --node $NODE --gas-prices 0.0055untrn --dry-run
```
Expected: two `gas estimate: …` lines. The migrate prints `to use new code: unauthorized` until the instantiate permission in PROPOSAL.md has been changed. To check that permission on its own:
```sh
neutrond q wasm code-info 5412 --node $NODE
# instantiate_permission: { permission: AnyOfAddresses, addresses: [ neutron1rtydk5vlppj2nmw98ctpgee6hxe5va7mc9x3lng8xm75p3dus80s7pkvmu ] }
```

## 5. After execution
```sh
neutrond q wasm contract $WM --node $NODE | grep code_id                                   # 5412
neutrond q wasm contract-state smart $WM '{"payout_ratio":{}}' --node $NODE                # "0.7121"
neutrond q wasm contract-state raw $WM 636f6e74726163745f696e666f --node $NODE -o json | jq -r .data | base64 -d   # version 1.0.1
neutrond q wasm contract-state smart $WM '{"config":{}}' --node $NODE                      # unchanged
neutrond q wasm contract-state smart $CONV '{"rate":[]}' --node $NODE                      # "0.9266621389"
neutrond q wasm contract-state smart $CONV '{"ownership":{}}' --node $NODE                  # owner $DAO
neutrond q wasm contract $WM --node $NODE | grep admin                                     # $FACTORY
neutrond q wasm contract $CONV --node $NODE | grep admin                                   # $DAO
```

## 6. Test transfers, then refill
1. Send a small amount of ATOM to each contract.
2. Converter: `swap` 1 dATOM. Expect `amount_out` = floor(1,000,000 × 0.9266621389) = 926,662 uatom.
3. WM: claim one small voucher. Expect `payout_amount` = floor(floor(share × unbonded) × 0.7121), with a `payout_ratio` = 0.7121 attribute in the event.
4. Top up the full amounts: WM 82,520 ATOM; converter 1,132,075 ATOM plus 735,337 dATOM.
