# Drop Admin proposal: dATOM recovery haircut

**DAO:** Drop Admin `neutron1xm4xgfv4xz4ccv0tjvlfac5gqwjnv9zzx4l47t7ve7j2sn4k7gwqkg947d`
**Submit via:** DAO DAO → New proposal → "Custom" action. Paste `proposal_dao_dao_custom.json`.

## Title
dATOM recovery: withdrawal manager haircut + converter rate

## Description
On 2026-09-22 the dATOM converter and withdrawal manager were drained. The converter held 1,589,711.92 ATOM and the withdrawal manager held 115,878.18 ATOM; both are empty now. About 71.21% of the ATOM is being recovered. This proposal splits that 71.21% equally across every remaining claim. Without it, claims would be paid first come, first served.

1. **Withdrawal manager** (`neutron1gckl…2uumck`): migrate from code 1728 to code **5412**, with `{"payout_ratio":"0.7121"}`. Every voucher claimed after this pays 71.21% of `user_share × unbonded_amount`, whatever order people claim in. Code 5412 is v1.0.0, the deployed source with checksum reproduced, plus the payout ratio, in commit `da87d2cf` on `fix/datom-wm-haircut`. It is built with `make compile_neutron` (commit `6a35c789`). Checksum: `04f4e59156c299405755a8d2b5a1daa2c91f29213719412f18716d1e2ac6420a`.
2. **Converter** (`neutron1502…4kqj9`): `update_rate` from 1.301309 to **0.9266621389** ATOM per dATOM, which is 1.301309 × 0.7121. Every circulating dATOM, LP holdings included, converts at the same reduced rate.

Neither contract is paused. Both stay non-functional until they are refilled: 82,520 ATOM to the withdrawal manager, and 1,132,075 ATOM plus 735,337 dATOM to the converter. Refilling happens after this proposal executes and after test transfers.

Verification steps: see VERIFY.md.

## Messages (decoded)
1. `wasm.execute` on factory `neutron1rtydk5vlppj2nmw98ctpgee6hxe5va7mc9x3lng8xm75p3dus80s7pkvmu`:
   `{"admin_execute":{"msgs":[{"wasm":{"migrate":{"contract_addr":"neutron1gcklauurx4ckyp0t8a6axl0500vy3n09y5utpspjg8ruxnln8erq2uumck","new_code_id":5412,"msg":"eyJwYXlvdXRfcmF0aW8iOiIwLjcxMjEifQ=="}}}]}}`
   (the inner `msg` base64-decodes to `{"payout_ratio":"0.7121"}`)
2. `wasm.execute` on converter `neutron15024sa6het7qrgccrx9wtxff7mh9qrcu9m6gvvlvez6em46ylscqk4kqj9`:
   `{"update_rate":{"rate":"0.9266621389"}}`

## Before the proposal executes
The uploader of code 5412 (`neutron1ygejj7rnheqlvvmcnmggllcd9y226ql598pkye`) must allow the factory to use the code. wasmd's migrate checks the new code's instantiate permission against the caller, which is the factory. Code 5412 is currently `Nobody`, so the migrate would fail:

```sh
neutrond tx wasm update-instantiate-config 5412 \
  --instantiate-anyof-addresses neutron1rtydk5vlppj2nmw98ctpgee6hxe5va7mc9x3lng8xm75p3dus80s7pkvmu \
  --from <uploader key> --chain-id neutron-1 --node <rpc> --gas auto --gas-adjustment 1.3 --gas-prices 0.0055untrn
```

The DAO's proposal module has `close_proposal_on_execution_failure: true`. If the proposal executes before this permission change, it is closed and has to be resubmitted.

## CLI alternative (members only, no deposit)
```sh
jq -n --arg t "<title>" --arg d "<description>" --slurpfile m proposal_msgs_onchain.json \
  '{propose:{msg:{propose:{title:$t,description:$d,msgs:$m[0]}}}}' > propose.json
neutrond tx wasm execute neutron12dl5dec8yg2jylqxyz5fmt9j2h7kg0gc8m5xg7e6ecpgwraxmadsgr08cc "$(cat propose.json)" \
  --from <member key> --chain-id neutron-1 --node <rpc> --gas auto --gas-adjustment 1.3 --gas-prices 0.0055untrn
```
