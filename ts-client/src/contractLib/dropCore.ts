import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
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
export type ContractState = "idle" | "peripheral" | "claiming" | "unbonding";
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
export type IBCTransferReason = "l_s_m_share" | "delegate";
export type String = string;
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
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal1 = string;
export type UnbondBatchStatus =
  | "new"
  | "unbond_requested"
  | "unbond_failed"
  | "unbonding"
  | "withdrawing"
  | "withdrawn"
  | "withdrawing_emergency"
  | "withdrawn_emergency";
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

export interface DropCoreSchema {
  responses:
    | ArrayOfString
    | ArrayOfAddr
    | Config
    | ContractState
    | Uint1281
    | Decimal
    | FailedBatchResponse
    | LastPuppeteerResponse
    | String
    | Pause
    | Uint1282
    | Uint1283
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
    | SetPauseArgs
    | SetBondHooksArgs
    | UpdateOwnershipArgs;
  instantiate?: InstantiateMsg;
  [k: string]: unknown;
}
export interface Config {
  base_denom: string;
  bond_limit?: Uint128 | null;
  emergency_address?: string | null;
  factory_contract: Addr;
  icq_update_delay: number;
  idle_min_interval: number;
  pump_ica_address?: string | null;
  remote_denom: string;
  transfer_channel_id: string;
  unbond_batch_switch_time: number;
  unbonding_period: number;
  unbonding_safe_period: number;
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
export interface Pause {
  bond: boolean;
  tick: boolean;
  unbond: boolean;
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
  factory_contract?: string | null;
  idle_min_interval?: number | null;
  pump_ica_address?: string | null;
  remote_denom?: string | null;
  rewards_receiver?: string | null;
  transfer_channel_id?: string | null;
  unbond_batch_switch_time?: number | null;
  unbonding_period?: number | null;
  unbonding_safe_period?: number | null;
}
export interface UpdateWithdrawnAmountArgs {
  batch_id: number;
  withdrawn_amount: Uint128;
}
export interface ProcessEmergencyBatchArgs {
  batch_id: number;
  unbonded_amount: Uint128;
}
export interface SetPauseArgs {
  type?: "object";
  required?: ["bond", "tick", "unbond"];
  properties?: {
    [k: string]: unknown;
  };
  additionalProperties?: never;
}
export interface SetBondHooksArgs {
  hooks: string[];
}
export interface InstantiateMsg {
  base_denom: string;
  bond_limit?: Uint128 | null;
  emergency_address?: string | null;
  factory_contract: string;
  icq_update_delay: number;
  idle_min_interval: number;
  owner: string;
  pump_ica_address?: string | null;
  remote_denom: string;
  transfer_channel_id: string;
  unbond_batch_switch_time: number;
  unbonding_period: number;
  unbonding_safe_period: number;
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
  queryCurrentUnbondBatch = async(): Promise<Uint128> => {
    return this.client.queryContractSmart(this.contractAddress, { current_unbond_batch: {} });
  }
  queryUnbondBatch = async(args: UnbondBatchArgs): Promise<UnbondBatch> => {
    return this.client.queryContractSmart(this.contractAddress, { unbond_batch: args });
  }
  queryUnbondBatches = async(args: UnbondBatchesArgs): Promise<UnbondBatchesResponse> => {
    return this.client.queryContractSmart(this.contractAddress, { unbond_batches: args });
  }
  queryContractState = async(): Promise<ContractState> => {
    return this.client.queryContractSmart(this.contractAddress, { contract_state: {} });
  }
  queryLastPuppeteerResponse = async(): Promise<LastPuppeteerResponse> => {
    return this.client.queryContractSmart(this.contractAddress, { last_puppeteer_response: {} });
  }
  queryTotalBonded = async(): Promise<Uint128> => {
    return this.client.queryContractSmart(this.contractAddress, { total_bonded: {} });
  }
  queryBondProviders = async(): Promise<ArrayOfAddr> => {
    return this.client.queryContractSmart(this.contractAddress, { bond_providers: {} });
  }
  queryTotalAsyncTokens = async(): Promise<Uint128> => {
    return this.client.queryContractSmart(this.contractAddress, { total_async_tokens: {} });
  }
  queryFailedBatch = async(): Promise<FailedBatchResponse> => {
    return this.client.queryContractSmart(this.contractAddress, { failed_batch: {} });
  }
  queryPause = async(): Promise<Pause> => {
    return this.client.queryContractSmart(this.contractAddress, { pause: {} });
  }
  queryBondHooks = async(): Promise<ArrayOfString> => {
    return this.client.queryContractSmart(this.contractAddress, { bond_hooks: {} });
  }
  bond = async(sender:string, args: BondArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { bond: args }, fee || "auto", memo, funds);
  }
  unbond = async(sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { unbond: {} }, fee || "auto", memo, funds);
  }
  tick = async(sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { tick: {} }, fee || "auto", memo, funds);
  }
  addBondProvider = async(sender:string, args: AddBondProviderArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { add_bond_provider: args }, fee || "auto", memo, funds);
  }
  removeBondProvider = async(sender:string, args: RemoveBondProviderArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { remove_bond_provider: args }, fee || "auto", memo, funds);
  }
  updateConfig = async(sender:string, args: UpdateConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_config: args }, fee || "auto", memo, funds);
  }
  updateWithdrawnAmount = async(sender:string, args: UpdateWithdrawnAmountArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_withdrawn_amount: args }, fee || "auto", memo, funds);
  }
  peripheralHook = async(sender:string, args: PeripheralHookArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { peripheral_hook: args }, fee || "auto", memo, funds);
  }
  resetBondedAmount = async(sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { reset_bonded_amount: {} }, fee || "auto", memo, funds);
  }
  processEmergencyBatch = async(sender:string, args: ProcessEmergencyBatchArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { process_emergency_batch: args }, fee || "auto", memo, funds);
  }
  setPause = async(sender:string, args: SetPauseArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { set_pause: args }, fee || "auto", memo, funds);
  }
  setBondHooks = async(sender:string, args: SetBondHooksArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { set_bond_hooks: args }, fee || "auto", memo, funds);
  }
  updateOwnership = async(sender:string, args: UpdateOwnershipArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_ownership: args }, fee || "auto", memo, funds);
  }
}
