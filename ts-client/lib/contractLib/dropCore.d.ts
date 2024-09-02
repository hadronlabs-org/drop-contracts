import {
  CosmWasmClient,
  SigningCosmWasmClient,
  ExecuteResult,
  InstantiateResult,
} from '@cosmjs/cosmwasm-stargate';
import { StdFee } from '@cosmjs/amino';
export type ArrayOfString = string[];
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
export type ArrayOfAddr = Addr[];
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
export type ContractState = 'idle' | 'peripheral' | 'claiming' | 'unbonding';
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
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string;
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
        real_amount: number;
        reason: IBCTransferReason;
        recipient: string;
      };
    }
  | {
      stake: {
        amount: Uint128;
      };
    }
  | {
      transfer: {
        interchain_account_id: string;
        items: [string, Coin][];
      };
    }
  | {
      setup_protocol: {
        interchain_account_id: string;
        rewards_withdraw_address: string;
      };
    };
export type IBCTransferReason = 'l_s_m_share' | 'delegate';
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
export type Uint1282 = string;
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
export type Uint1283 = string;
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
export type Uint1284 = string;
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal1 = string;
export type UnbondBatchStatus =
  | 'new'
  | 'unbond_requested'
  | 'unbond_failed'
  | 'unbonding'
  | 'withdrawing'
  | 'withdrawn'
  | 'withdrawing_emergency'
  | 'withdrawn_emergency';
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
export type PeripheralHookArgs =
  | {
      success: ResponseHookSuccessMsg;
    }
  | {
      error: ResponseHookErrorMsg;
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
  | 'accept_ownership'
  | 'renounce_ownership';
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
export interface DropCoreSchema {
  responses:
    | ArrayOfAddr
    | Config
    | ContractState
    | Uint1281
    | Decimal
    | FailedBatchResponse
    | LastPuppeteerResponse
    | String
    | PauseInfoResponse
    | Uint1282
    | Uint1283
    | Uint1284
    | UnbondBatch
    | UnbondBatchesResponse;
  query: UnbondBatchArgs | UnbondBatchesArgs;
  execute:
    | BondArgs
    | AddBondProviderArgs
    | RemoveBondProviderArgs
    | UpdateConfigArgs
    | UpdateWithdrawnAmountArgs
    | PeripheralHookArgs
    | ProcessEmergencyBatchArgs
    | UpdateOwnershipArgs;
  instantiate?: InstantiateMsg;
  [k: string]: unknown;
}
export interface Config {
  base_denom: string;
  bond_limit?: Uint128 | null;
  emergency_address?: string | null;
  icq_update_delay: number;
  idle_min_interval: number;
  pump_ica_address?: string | null;
  puppeteer_contract: Addr;
  remote_denom: string;
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
export interface FailedBatchResponse {
  response?: number | null;
}
export interface LastPuppeteerResponse {
  response?: ResponseHookMsg | null;
}
export interface ResponseHookSuccessMsg {
  answers: ResponseAnswer[];
  local_height: number;
  remote_height: number;
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
export interface UnbondBatch {
  expected_native_asset_amount: Uint128;
  expected_release_time: number;
  slashing_effect?: Decimal1 | null;
  status: UnbondBatchStatus;
  status_timestamps: UnbondBatchStatusTimestamps;
  total_dasset_amount_to_withdraw: Uint128;
  total_unbond_items: number;
  unbonded_amount?: Uint128 | null;
  withdrawn_amount?: Uint128 | null;
}
export interface UnbondBatchStatusTimestamps {
  new: number;
  unbond_failed?: number | null;
  unbond_requested?: number | null;
  unbonding?: number | null;
  withdrawing?: number | null;
  withdrawing_emergency?: number | null;
  withdrawn?: number | null;
  withdrawn_emergency?: number | null;
}
export interface UnbondBatchesResponse {
  next_page_key?: Uint128 | null;
  unbond_batches: UnbondBatch1[];
}
export interface UnbondBatch1 {
  expected_native_asset_amount: Uint128;
  expected_release_time: number;
  slashing_effect?: Decimal1 | null;
  status: UnbondBatchStatus;
  status_timestamps: UnbondBatchStatusTimestamps;
  total_dasset_amount_to_withdraw: Uint128;
  total_unbond_items: number;
  unbonded_amount?: Uint128 | null;
  withdrawn_amount?: Uint128 | null;
}
export interface UnbondBatchArgs {
  batch_id: Uint128;
}
export interface UnbondBatchesArgs {
  limit?: Uint64 | null;
  page_key?: Uint128 | null;
}
export interface BondArgs {
  receiver?: string | null;
  ref?: string | null;
}
export interface AddBondProviderArgs {
  bond_provider_address: string;
}
export interface RemoveBondProviderArgs {
  bond_provider_address: string;
}
export interface UpdateConfigArgs {
  new_config: ConfigOptional;
}
export interface ConfigOptional {
  base_denom?: string | null;
  bond_limit?: Uint128 | null;
  emergency_address?: string | null;
  idle_min_interval?: number | null;
  pump_ica_address?: string | null;
  puppeteer_contract?: string | null;
  remote_denom?: string | null;
  rewards_receiver?: string | null;
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
export interface UpdateWithdrawnAmountArgs {
  batch_id: number;
  withdrawn_amount: Uint128;
}
export interface ProcessEmergencyBatchArgs {
  batch_id: number;
  unbonded_amount: Uint128;
}
export interface SetBondHooksArgs {
  hooks: string[];
}
export interface InstantiateMsg {
  base_denom: string;
  bond_limit?: Uint128 | null;
  emergency_address?: string | null;
  icq_update_delay: number;
  idle_min_interval: number;
  owner: string;
  pump_ica_address?: string | null;
  puppeteer_contract: string;
  remote_denom: string;
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
export declare class Client {
  private readonly client;
  contractAddress: string;
  constructor(
    client: CosmWasmClient | SigningCosmWasmClient,
    contractAddress: string,
  );
  mustBeSigningClient(): Error;
  static instantiate(
    client: SigningCosmWasmClient,
    sender: string,
    codeId: number,
    initMsg: InstantiateMsg,
    label: string,
    fees: StdFee | 'auto' | number,
    initCoins?: readonly Coin[],
  ): Promise<InstantiateResult>;
  static instantiate2(
    client: SigningCosmWasmClient,
    sender: string,
    codeId: number,
    salt: number,
    initMsg: InstantiateMsg,
    label: string,
    fees: StdFee | 'auto' | number,
    initCoins?: readonly Coin[],
  ): Promise<InstantiateResult>;
  queryConfig: () => Promise<Config>;
  queryOwner: () => Promise<String>;
  queryExchangeRate: () => Promise<Decimal>;
  queryCurrentUnbondBatch: () => Promise<Uint128>;
  queryUnbondBatch: (args: UnbondBatchArgs) => Promise<UnbondBatch>;
  queryUnbondBatches: (
    args: UnbondBatchesArgs,
  ) => Promise<UnbondBatchesResponse>;
  queryContractState: () => Promise<ContractState>;
  queryLastPuppeteerResponse: () => Promise<LastPuppeteerResponse>;
  queryTotalBonded: () => Promise<Uint128>;
  queryBondProviders: () => Promise<ArrayOfAddr>;
  queryTotalLSMShares: () => Promise<Uint128>;
  queryTotalAsyncTokens: () => Promise<Uint128>;
  queryFailedBatch: () => Promise<FailedBatchResponse>;
  queryBondHooks: () => Promise<ArrayOfString>;
  queryPauseInfo: () => Promise<PauseInfoResponse>;
  bond: (
    sender: string,
    args: BondArgs,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>;
  unbond: (
    sender: string,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>;
  addBondProvider: (
    sender: string,
    args: AddBondProviderArgs,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>;
  removeBondProvider: (
    sender: string,
    args: RemoveBondProviderArgs,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>;
  updateConfig: (
    sender: string,
    args: UpdateConfigArgs,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>;
  updateWithdrawnAmount: (
    sender: string,
    args: UpdateWithdrawnAmountArgs,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>;
  tick: (
    sender: string,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>;
  peripheralHook: (
    sender: string,
    args: PeripheralHookArgs,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>;
  resetBondedAmount: (
    sender: string,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>;
  processEmergencyBatch: (
    sender: string,
    args: ProcessEmergencyBatchArgs,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>;
  setBondHooks: (
    sender: string,
    args: SetBondHooksArgs,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>;
  pause: (
    sender: string,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>;
  unpause: (
    sender: string,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>;
  updateOwnership: (
    sender: string,
    args: UpdateOwnershipArgs,
    fee?: number | StdFee | 'auto',
    memo?: string,
    funds?: Coin[],
  ) => Promise<ExecuteResult>;
}
