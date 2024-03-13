import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string;

export interface InstantiateMsg {
  base_denom: string;
  channel: string;
  fee?: Decimal | null;
  fee_address?: string | null;
  idle_min_interval: number;
  lsm_redeem_threshold: number;
  owner: string;
  pump_address?: string | null;
  puppeteer_contract: string;
  puppeteer_timeout: number;
  remote_denom: string;
  strategy_contract: string;
  token_contract: string;
  unbond_batch_switch_time: number;
  unbonding_period: number;
  unbonding_safe_period: number;
  validators_set_contract: string;
  withdrawal_manager_contract: string;
  withdrawal_voucher_contract: string;
}
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string;
export type ContractState = "idle" | "claiming" | "unbonding" | "staking" | "transfering";
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal1 = string;
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
export type ArrayOfTupleOf_StringAnd_TupleOf_StringAnd_Uint128 = [string, [string, Uint128]][];
export type ResponseHookMsg =
  | {
      success: ResponseHookSuccessMsg;
    }
  | {
      error: ResponseHookErrorMsg;
    };
export type ResponseAnswer =
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
        transfer?: TransferReadyBatchMsg | null;
        validators: string[];
      };
    }
  | {
      i_b_c_transfer: {
        amount: number;
        denom: string;
        recipient: string;
      };
    }
  | {
      transfer: {
        interchain_account_id: string;
        items: [string, Coin][];
      };
    };
export type ArrayOfNonNativeRewardsItem = NonNativeRewardsItem[];
export type String = string;
export type ArrayOfTupleOf_StringAnd_TupleOf_StringAnd_Uint1281 = [string, [string, Uint128]][];
export type UnbondBatchStatus =
  | "new"
  | "unbond_requested"
  | "unbond_failed"
  | "unbonding"
  | "unbonded"
  | "withdrawing"
  | "withdrawn";
export type PuppeteerHookArgs =
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

export interface LidoCoreSchema {
  responses:
    | Config
    | ContractState
    | Decimal1
    | ArrayOfTupleOf_StringAnd_TupleOf_StringAnd_Uint128
    | ResponseHookMsg
    | ArrayOfNonNativeRewardsItem
    | String
    | ArrayOfTupleOf_StringAnd_TupleOf_StringAnd_Uint1281
    | UnbondBatch;
  query: UnbondBatchArgs;
  execute: BondArgs | UpdateConfigArgs | UpdateNonNativeRewardsReceiversArgs | PuppeteerHookArgs | UpdateOwnershipArgs;
  [k: string]: unknown;
}
export interface Config {
  base_denom: string;
  channel: string;
  fee?: Decimal | null;
  fee_address?: string | null;
  idle_min_interval: number;
  ld_denom?: string | null;
  lsm_redeem_threshold: number;
  pump_address?: string | null;
  puppeteer_contract: string;
  puppeteer_timeout: number;
  remote_denom: string;
  strategy_contract: string;
  token_contract: string;
  unbond_batch_switch_time: number;
  unbonding_period: number;
  unbonding_safe_period: number;
  validators_set_contract: string;
  withdrawal_manager_contract: string;
  withdrawal_voucher_contract: string;
}
export interface ResponseHookSuccessMsg {
  answers: ResponseAnswer[];
  request: RequestPacket;
  request_id: number;
  transaction: Transaction;
}
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
export interface TransferReadyBatchMsg {
  amount: Uint128;
  batch_id: number;
  recipient: string;
}
export interface ResponseHookErrorMsg {
  details: string;
  request: RequestPacket;
  request_id: number;
  transaction: Transaction;
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
  unbond_items: UnbondItem[];
  unbonded_amount?: Uint128 | null;
  withdrawed_amount?: Uint128 | null;
}
export interface UnbondItem {
  amount: Uint128;
  expected_amount: Uint128;
  sender: string;
}
export interface UnbondBatchArgs {
  batch_id: Uint128;
}
export interface BondArgs {
  receiver?: string | null;
}
export interface UpdateConfigArgs {
  new_config: ConfigOptional;
}
export interface ConfigOptional {
  base_denom?: string | null;
  channel?: string | null;
  fee?: Decimal | null;
  fee_address?: string | null;
  idle_min_interval?: number | null;
  ld_denom?: string | null;
  lsm_redeem_threshold?: number | null;
  pump_address?: string | null;
  puppeteer_contract?: string | null;
  puppeteer_timeout?: number | null;
  remote_denom?: string | null;
  strategy_contract?: string | null;
  token_contract?: string | null;
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
    initCoins?: readonly Coin[],
    fees?: StdFee | 'auto' | number,
  ): Promise<InstantiateResult> {
    const res = await client.instantiate(sender, codeId, initMsg, label, fees, {
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
  queryLastPuppeteerResponse = async(): Promise<ResponseHookMsg> => {
    return this.client.queryContractSmart(this.contractAddress, { last_puppeteer_response: {} });
  }
  queryNonNativeRewardsReceivers = async(): Promise<ArrayOfNonNativeRewardsItem> => {
    return this.client.queryContractSmart(this.contractAddress, { non_native_rewards_receivers: {} });
  }
  queryPendingLSMShares = async(): Promise<ArrayOfTupleOf_StringAnd_Tuple_of_String_and_Uint128> => {
    return this.client.queryContractSmart(this.contractAddress, { pending_l_s_m_shares: {} });
  }
  queryLSMSharesToRedeem = async(): Promise<ArrayOfTupleOf_StringAnd_Tuple_of_String_and_Uint128> => {
    return this.client.queryContractSmart(this.contractAddress, { l_s_m_shares_to_redeem: {} });
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
  updateOwnership = async(sender:string, args: UpdateOwnershipArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_ownership: args }, fee || "auto", memo, funds);
  }
}
