import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
/**
 * A thin wrapper around u128 that is using strings for JSON encoding/decoding, such that the full u128 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u128` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint128; let a = Uint128::from(123u128); assert_eq!(a.u128(), 123);
 *
 * let b = Uint128::from(42u64); assert_eq!(b.u128(), 42);
 *
 * let c = Uint128::from(70u32); assert_eq!(c.u128(), 70); ```
 */
export type Uint128 = string;
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string;
/**
 * A human readable address.
 *
 * In Cosmos, this is typically bech32 encoded. But for multi-chain smart contracts no assumptions should be made other than being UTF-8 encoded and of reasonable length.
 *
 * This type represents a validated address. It can be created in the following ways 1. Use `Addr::unchecked(input)` 2. Use `let checked: Addr = deps.api.addr_validate(input)?` 3. Use `let checked: Addr = deps.api.addr_humanize(canonical_addr)?` 4. Deserialize from JSON. This must only be done from JSON that was validated before such as a contract's state. `Addr` must not be used in messages sent by the user because this would result in unvalidated instances.
 *
 * This type is immutable. If you really need to mutate it (Really? Are you sure?), create a mutable copy using `let mut mutable = Addr::to_string()` and operate on that `String` instance.
 */
export type Addr = string;
export type ContractState = "idle" | "claiming" | "unbonding" | "staking_rewards" | "staking_bond";
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal1 = string;
export type ArrayOfTupleOfStringAndTupleOfStringAndUint128 = [string, [string, Uint128]][];
export type ResponseHookMsg =
  | {
      success: ResponseHookSuccessMsg;
    }
  | {
      error: ResponseHookErrorMsg;
    };
export type ResponseAnswer =
  | {
      grant_delegate_response: MsgGrantResponse;
    }
  | {
      delegate_response: MsgDelegateResponse;
    }
  | {
      undelegate_response: MsgUndelegateResponse;
    }
  | {
      begin_redelegate_response: MsgBeginRedelegateResponse;
    }
  | {
      tokenize_shares_response: MsgTokenizeSharesResponse;
    }
  | {
      redeem_tokensfor_shares_response: MsgRedeemTokensforSharesResponse;
    }
  | {
      authz_exec_response: MsgExecResponse;
    }
  | {
      i_b_c_transfer: MsgIBCTransfer;
    }
  | {
      transfer_response: MsgSendResponse;
    }
  | {
      unknown_response: {};
    };
/**
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 *
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>. See also <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
 */
export type Binary = string;
export type Transaction =
  | {
      delegate: {
        denom: string;
        interchain_account_id: string;
        items: [string, Uint128][];
      };
    }
  | {
      undelegate: {
        batch_id: number;
        denom: string;
        interchain_account_id: string;
        items: [string, Uint128][];
      };
    }
  | {
      redelegate: {
        amount: number;
        denom: string;
        interchain_account_id: string;
        validator_from: string;
        validator_to: string;
      };
    }
  | {
      withdraw_reward: {
        interchain_account_id: string;
        validator: string;
      };
    }
  | {
      tokenize_share: {
        amount: number;
        denom: string;
        interchain_account_id: string;
        validator: string;
      };
    }
  | {
      redeem_shares: {
        interchain_account_id: string;
        items: RedeemShareItem[];
      };
    }
  | {
      claim_rewards_and_optionaly_transfer: {
        denom: string;
        interchain_account_id: string;
        transfer?: TransferReadyBatchesMsg | null;
        validators: string[];
      };
    }
  | {
      i_b_c_transfer: {
        amount: number;
        denom: string;
        reason: IBCTransferReason;
        recipient: string;
      };
    }
  | {
      transfer: {
        interchain_account_id: string;
        items: [string, Coin][];
      };
    }
  | {
      grant_delegate: {
        grantee: string;
        interchain_account_id: string;
      };
    };
export type IBCTransferReason = "l_s_m_share" | "stake";
export type ArrayOfNonNativeRewardsItem = NonNativeRewardsItem[];
export type String = string;
/**
 * Information about if the contract is currently paused.
 */
export type PauseInfoResponse =
  | {
      paused: {};
    }
  | {
      unpaused: {};
    };
export type ArrayOfTupleOfStringAndTupleOfStringAndUint1281 = [string, [string, Uint128]][];
/**
 * A thin wrapper around u128 that is using strings for JSON encoding/decoding, such that the full u128 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u128` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint128; let a = Uint128::from(123u128); assert_eq!(a.u128(), 123);
 *
 * let b = Uint128::from(42u64); assert_eq!(b.u128(), 42);
 *
 * let c = Uint128::from(70u32); assert_eq!(c.u128(), 70); ```
 */
export type Uint1281 = string;
export type UnbondBatchStatus =
  | "new"
  | "unbond_requested"
  | "unbond_failed"
  | "unbonding"
  | "withdrawing"
  | "withdrawn"
  | "withdrawing_emergency"
  | "withdrawn_emergency";
export type PuppeteerHookArgs =
  | {
      success: ResponseHookSuccessMsg;
    }
  | {
      error: ResponseHookErrorMsg;
    };
export type StakerHookArgs =
  | {
      success: ResponseHookSuccessMsg2;
    }
  | {
      error: ResponseHookErrorMsg2;
    };
export type Transaction2 =
  | {
      stake: {
        amount: Uint128;
      };
    }
  | {
      i_b_c_transfer: {
        amount: Uint128;
      };
    };
/**
 * Actions that can be taken to alter the contract's ownership
 */
export type UpdateOwnershipArgs =
  | {
      transfer_ownership: {
        expiry?: Expiration | null;
        new_owner: string;
      };
    }
  | "accept_ownership"
  | "renounce_ownership";
/**
 * Expiration represents a point in time when some event happens. It can compare with a BlockInfo and will return is_expired() == true once the condition is hit (and for every block in the future)
 */
export type Expiration =
  | {
      at_height: number;
    }
  | {
      at_time: Timestamp2;
    }
  | {
      never: {};
    };
/**
 * A point in time in nanosecond precision.
 *
 * This type can represent times from 1970-01-01T00:00:00Z to 2554-07-21T23:34:33Z.
 *
 * ## Examples
 *
 * ``` # use cosmwasm_std::Timestamp; let ts = Timestamp::from_nanos(1_000_000_202); assert_eq!(ts.nanos(), 1_000_000_202); assert_eq!(ts.seconds(), 1); assert_eq!(ts.subsec_nanos(), 202);
 *
 * let ts = ts.plus_seconds(2); assert_eq!(ts.nanos(), 3_000_000_202); assert_eq!(ts.seconds(), 3); assert_eq!(ts.subsec_nanos(), 202); ```
 */
export type Timestamp2 = Uint64;
/**
 * A thin wrapper around u64 that is using strings for JSON encoding/decoding, such that the full u64 range can be used for clients that convert JSON numbers to floats, like JavaScript and jq.
 *
 * # Examples
 *
 * Use `from` to create instances of this and `u64` to get the value out:
 *
 * ``` # use cosmwasm_std::Uint64; let a = Uint64::from(42u64); assert_eq!(a.u64(), 42);
 *
 * let b = Uint64::from(70u32); assert_eq!(b.u64(), 70); ```
 */
export type Uint64 = string;

export interface DropCoreSchema {
  responses:
    | Config
    | ContractState
    | Decimal1
    | ArrayOfTupleOfStringAndTupleOfStringAndUint128
    | LastPuppeteerResponse
    | LastStakerResponse
    | ArrayOfNonNativeRewardsItem
    | String
    | PauseInfoResponse
    | ArrayOfTupleOfStringAndTupleOfStringAndUint1281
    | Uint1281
    | UnbondBatch;
  query: UnbondBatchArgs;
  execute:
    | BondArgs
    | UpdateConfigArgs
    | UpdateNonNativeRewardsReceiversArgs
    | PuppeteerHookArgs
    | StakerHookArgs
    | ProcessEmergencyBatchArgs
    | UpdateOwnershipArgs;
  instantiate?: InstantiateMsg;
  [k: string]: unknown;
}
export interface Config {
  base_denom: string;
  bond_limit?: Uint128 | null;
  emergency_address?: string | null;
  fee?: Decimal | null;
  fee_address?: string | null;
  idle_min_interval: number;
  lsm_min_bond_amount: Uint128;
  lsm_redeem_maximum_interval: number;
  lsm_redeem_threshold: number;
  min_stake_amount: Uint128;
  pump_ica_address?: string | null;
  puppeteer_contract: Addr;
  puppeteer_timeout: number;
  remote_denom: string;
  staker_contract: Addr;
  strategy_contract: Addr;
  token_contract: Addr;
  transfer_channel_id: string;
  unbond_batch_switch_time: number;
  unbonding_period: number;
  unbonding_safe_period: number;
  validators_set_contract: Addr;
  withdrawal_manager_contract: Addr;
  withdrawal_voucher_contract: Addr;
}
export interface LastPuppeteerResponse {
  response?: ResponseHookMsg | null;
}
export interface ResponseHookSuccessMsg {
  answers: ResponseAnswer[];
  local_height: number;
  request: RequestPacket;
  request_id: number;
  transaction: Transaction;
}
export interface MsgGrantResponse {}
export interface MsgDelegateResponse {}
export interface MsgUndelegateResponse {
  completion_time?: Timestamp | null;
}
export interface Timestamp {
  nanos: number;
  seconds: number;
}
export interface MsgBeginRedelegateResponse {
  completion_time?: Timestamp | null;
}
export interface MsgTokenizeSharesResponse {
  amount?: Coin | null;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export interface MsgRedeemTokensforSharesResponse {
  amount?: Coin | null;
}
export interface MsgExecResponse {
  results: number[][];
}
export interface MsgIBCTransfer {}
export interface MsgSendResponse {}
export interface RequestPacket {
  data?: Binary | null;
  destination_channel?: string | null;
  destination_port?: string | null;
  sequence?: number | null;
  source_channel?: string | null;
  source_port?: string | null;
  timeout_height?: RequestPacketTimeoutHeight | null;
  timeout_timestamp?: number | null;
  [k: string]: unknown;
}
export interface RequestPacketTimeoutHeight {
  revision_height?: number | null;
  revision_number?: number | null;
  [k: string]: unknown;
}
export interface RedeemShareItem {
  amount: Uint128;
  local_denom: string;
  remote_denom: string;
}
export interface TransferReadyBatchesMsg {
  amount: Uint128;
  batch_ids: number[];
  emergency: boolean;
  recipient: string;
}
export interface ResponseHookErrorMsg {
  details: string;
  request: RequestPacket;
  request_id: number;
  transaction: Transaction;
}
export interface LastStakerResponse {
  response?: ResponseHookMsg | null;
}
export interface NonNativeRewardsItem {
  address: string;
  denom: string;
  fee: Decimal;
  fee_address: string;
  min_amount: Uint128;
}
export interface UnbondBatch {
  created: number;
  expected_amount: Uint128;
  expected_release: number;
  slashing_effect?: Decimal | null;
  status: UnbondBatchStatus;
  total_amount: Uint128;
  total_unbond_items: number;
  unbonded_amount?: Uint128 | null;
  withdrawed_amount?: Uint128 | null;
}
export interface UnbondBatchArgs {
  batch_id: Uint128;
}
export interface BondArgs {
  receiver?: string | null;
  ref?: string | null;
}
export interface UpdateConfigArgs {
  new_config: ConfigOptional;
}
export interface ConfigOptional {
  base_denom?: string | null;
  bond_limit?: Uint128 | null;
  emergency_address?: string | null;
  fee?: Decimal | null;
  fee_address?: string | null;
  idle_min_interval?: number | null;
  lsm_min_bond_amount?: Uint128 | null;
  lsm_redeem_maximum_interval?: number | null;
  lsm_redeem_threshold?: number | null;
  min_stake_amount?: Uint128 | null;
  pump_ica_address?: string | null;
  puppeteer_contract?: string | null;
  puppeteer_timeout?: number | null;
  remote_denom?: string | null;
  staker_contract?: string | null;
  strategy_contract?: string | null;
  token_contract?: string | null;
  transfer_channel_id?: string | null;
  unbond_batch_switch_time?: number | null;
  unbonding_period?: number | null;
  unbonding_safe_period?: number | null;
  validators_set_contract?: string | null;
  withdrawal_manager_contract?: string | null;
  withdrawal_voucher_contract?: string | null;
}
export interface UpdateNonNativeRewardsReceiversArgs {
  items: NonNativeRewardsItem[];
}
export interface ResponseHookSuccessMsg2 {
  local_height: number;
  request: RequestPacket;
  request_id: number;
  transaction: Transaction2;
}
export interface ResponseHookErrorMsg2 {
  details: string;
  request: RequestPacket;
  request_id: number;
  transaction: Transaction2;
}
export interface ProcessEmergencyBatchArgs {
  batch_id: number;
  unbonded_amount: Uint128;
}
export interface InstantiateMsg {
  base_denom: string;
  bond_limit?: Uint128 | null;
  emergency_address?: string | null;
  fee?: Decimal | null;
  fee_address?: string | null;
  idle_min_interval: number;
  lsm_min_bond_amount: Uint128;
  lsm_redeem_max_interval: number;
  lsm_redeem_threshold: number;
  min_stake_amount: Uint128;
  owner: string;
  pump_ica_address?: string | null;
  puppeteer_contract: string;
  puppeteer_timeout: number;
  remote_denom: string;
  staker_contract: string;
  strategy_contract: string;
  token_contract: string;
  transfer_channel_id: string;
  unbond_batch_switch_time: number;
  unbonding_period: number;
  unbonding_safe_period: number;
  validators_set_contract: string;
  withdrawal_manager_contract: string;
  withdrawal_voucher_contract: string;
}


function isSigningCosmWasmClient(
  client: CosmWasmClient | SigningCosmWasmClient
): client is SigningCosmWasmClient {
  return 'execute' in client;
}

export class Client {
  private readonly client: CosmWasmClient | SigningCosmWasmClient;
  contractAddress: string;
  constructor(client: CosmWasmClient | SigningCosmWasmClient, contractAddress: string) {
    this.client = client;
    this.contractAddress = contractAddress;
  }
  mustBeSigningClient() {
    return new Error("This client is not a SigningCosmWasmClient");
  }
  static async instantiate(
    client: SigningCosmWasmClient,
    sender: string,
    codeId: number,
    initMsg: InstantiateMsg,
    label: string,
    fees: StdFee | 'auto' | number,
    initCoins?: readonly Coin[],
  ): Promise<InstantiateResult> {
    const res = await client.instantiate(sender, codeId, initMsg, label, fees, {
      ...(initCoins && initCoins.length && { funds: initCoins }),
    });
    return res;
  }
  static async instantiate2(
    client: SigningCosmWasmClient,
    sender: string,
    codeId: number,
    salt: number,
    initMsg: InstantiateMsg,
    label: string,
    fees: StdFee | 'auto' | number,
    initCoins?: readonly Coin[],
  ): Promise<InstantiateResult> {
    const res = await client.instantiate2(sender, codeId, new Uint8Array([salt]), initMsg, label, fees, {
      ...(initCoins && initCoins.length && { funds: initCoins }),
    });
    return res;
  }
  queryConfig = async(): Promise<Config> => {
    return this.client.queryContractSmart(this.contractAddress, { config: {} });
  }
  queryOwner = async(): Promise<String> => {
    return this.client.queryContractSmart(this.contractAddress, { owner: {} });
  }
  queryExchangeRate = async(): Promise<Decimal> => {
    return this.client.queryContractSmart(this.contractAddress, { exchange_rate: {} });
  }
  queryUnbondBatch = async(args: UnbondBatchArgs): Promise<UnbondBatch> => {
    return this.client.queryContractSmart(this.contractAddress, { unbond_batch: args });
  }
  queryContractState = async(): Promise<ContractState> => {
    return this.client.queryContractSmart(this.contractAddress, { contract_state: {} });
  }
  queryLastPuppeteerResponse = async(): Promise<LastPuppeteerResponse> => {
    return this.client.queryContractSmart(this.contractAddress, { last_puppeteer_response: {} });
  }
  queryLastStakerResponse = async(): Promise<LastStakerResponse> => {
    return this.client.queryContractSmart(this.contractAddress, { last_staker_response: {} });
  }
  queryNonNativeRewardsReceivers = async(): Promise<ArrayOfNonNativeRewardsItem> => {
    return this.client.queryContractSmart(this.contractAddress, { non_native_rewards_receivers: {} });
  }
  queryPendingLSMShares = async(): Promise<ArrayOfTupleOfStringAndTupleOfStringAndUint128> => {
    return this.client.queryContractSmart(this.contractAddress, { pending_l_s_m_shares: {} });
  }
  queryLSMSharesToRedeem = async(): Promise<ArrayOfTupleOfStringAndTupleOfStringAndUint128> => {
    return this.client.queryContractSmart(this.contractAddress, { l_s_m_shares_to_redeem: {} });
  }
  queryTotalBonded = async(): Promise<Uint128> => {
    return this.client.queryContractSmart(this.contractAddress, { total_bonded: {} });
  }
  queryPauseInfo = async(): Promise<PauseInfoResponse> => {
    return this.client.queryContractSmart(this.contractAddress, { pause_info: {} });
  }
  bond = async(sender:string, args: BondArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { bond: args }, fee || "auto", memo, funds);
  }
  unbond = async(sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { unbond: {} }, fee || "auto", memo, funds);
  }
  updateConfig = async(sender:string, args: UpdateConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_config: args }, fee || "auto", memo, funds);
  }
  updateNonNativeRewardsReceivers = async(sender:string, args: UpdateNonNativeRewardsReceiversArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_non_native_rewards_receivers: args }, fee || "auto", memo, funds);
  }
  tick = async(sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { tick: {} }, fee || "auto", memo, funds);
  }
  puppeteerHook = async(sender:string, args: PuppeteerHookArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { puppeteer_hook: args }, fee || "auto", memo, funds);
  }
  stakerHook = async(sender:string, args: StakerHookArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { staker_hook: args }, fee || "auto", memo, funds);
  }
  resetBondedAmount = async(sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { reset_bonded_amount: {} }, fee || "auto", memo, funds);
  }
  processEmergencyBatch = async(sender:string, args: ProcessEmergencyBatchArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { process_emergency_batch: args }, fee || "auto", memo, funds);
  }
  pause = async(sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { pause: {} }, fee || "auto", memo, funds);
  }
  unpause = async(sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { unpause: {} }, fee || "auto", memo, funds);
  }
  updateOwnership = async(sender:string, args: UpdateOwnershipArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_ownership: args }, fee || "auto", memo, funds);
  }
}
