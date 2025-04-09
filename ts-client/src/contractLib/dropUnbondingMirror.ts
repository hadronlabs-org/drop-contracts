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
export type ArrayOfFailedReceiverResponse = FailedReceiverResponse[];
export type NullableFailedReceiverResponse = FailedReceiverResponse | null;
/**
 * Expiration represents a point in time when some event happens. It can compare with a BlockInfo and will return is_expired() == true once the condition is hit (and for every block in the future)
 */
export type Expiration =
  | {
      at_height: number;
    }
  | {
      at_time: Timestamp;
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
export type Timestamp = Uint64;
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
export type Boolean = boolean;
export type String = string;
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

export interface DropUnbondingMirrorSchema {
  responses:
    | ArrayOfFailedReceiverResponse
    | Config
    | NullableFailedReceiverResponse
    | OwnershipForString
    | Boolean
    | String;
  query: FailedReceiverArgs | UnbondReadyArgs | VoucherToNftArgs;
  execute: UpdateConfigArgs | WithdrawArgs | UnbondArgs | RetryArgs | UpdateOwnershipArgs;
  instantiate?: InstantiateMsg;
  [k: string]: unknown;
}
export interface FailedReceiverResponse {
  failed_transfers: Coin[];
  receiver: string;
}
export interface Coin {
  amount: Uint128;
  denom: string;
}
export interface Config {
  core_contract: string;
  ibc_timeout: number;
  prefix: string;
  source_channel: string;
  source_port: string;
  withdrawal_manager: string;
  withdrawal_voucher: string;
}
/**
 * The contract's ownership info
 */
export interface OwnershipForString {
  /**
   * The contract's current owner. `None` if the ownership has been renounced.
   */
  owner?: string | null;
  /**
   * The deadline for the pending owner to accept the ownership. `None` if there isn't a pending ownership transfer, or if a transfer exists and it doesn't have a deadline.
   */
  pending_expiry?: Expiration | null;
  /**
   * The account who has been proposed to take over the ownership. `None` if there isn't a pending ownership transfer.
   */
  pending_owner?: string | null;
}
export interface FailedReceiverArgs {
  receiver: string;
}
export interface UnbondReadyArgs {
  id: string;
}
export interface VoucherToNftArgs {
  id: string;
}
export interface UpdateConfigArgs {
  new_config: ConfigOptional;
}
export interface ConfigOptional {
  core_contract?: string | null;
  ibc_timeout?: number | null;
  prefix?: string | null;
  source_channel?: string | null;
  source_port?: string | null;
  withdrawal_manager?: string | null;
  withdrawal_voucher?: string | null;
}
export interface WithdrawArgs {
  receiver: string;
}
export interface UnbondArgs {
  receiver: string;
}
export interface RetryArgs {
  receiver: string;
}
export interface InstantiateMsg {
  core_contract: string;
  ibc_timeout: number;
  owner?: string | null;
  prefix: string;
  source_channel: string;
  source_port: string;
  withdrawal_manager: string;
  withdrawal_voucher: string;
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
  mustBeSigningClient(): Error {
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
    admin?: string,
  ): Promise<InstantiateResult> {
    const res = await client.instantiate(sender, codeId, initMsg, label, fees, {
      ...(initCoins && initCoins.length && { funds: initCoins }), ...(admin && { admin: admin }),
    });
    return res;
  }
  static async instantiate2(
    client: SigningCosmWasmClient,
    sender: string,
    codeId: number,
    salt: Uint8Array,
    initMsg: InstantiateMsg,
    label: string,
    fees: StdFee | 'auto' | number,
    initCoins?: readonly Coin[],
    admin?: string,
  ): Promise<InstantiateResult> {
    const res = await client.instantiate2(sender, codeId, salt, initMsg, label, fees, {
      ...(initCoins && initCoins.length && { funds: initCoins }), ...(admin && { admin: admin }),
    });
    return res;
  }
  queryConfig = async(): Promise<Config> => {
    return this.client.queryContractSmart(this.contractAddress, { config: {} });
  }
  queryFailedReceiver = async(args: FailedReceiverArgs): Promise<NullableFailedReceiverResponse> => {
    return this.client.queryContractSmart(this.contractAddress, { failed_receiver: args });
  }
  queryAllFailed = async(): Promise<ArrayOfFailedReceiverResponse> => {
    return this.client.queryContractSmart(this.contractAddress, { all_failed: {} });
  }
  queryUnbondReady = async(args: UnbondReadyArgs): Promise<Boolean> => {
    return this.client.queryContractSmart(this.contractAddress, { unbond_ready: args });
  }
  queryVoucherToNft = async(args: VoucherToNftArgs): Promise<String> => {
    return this.client.queryContractSmart(this.contractAddress, { voucher_to_nft: args });
  }
  queryOwnership = async(): Promise<OwnershipForString> => {
    return this.client.queryContractSmart(this.contractAddress, { ownership: {} });
  }
  updateConfig = async(sender:string, args: UpdateConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, this.updateConfigMsg(args), fee || "auto", memo, funds);
  }
  updateConfigMsg = (args: UpdateConfigArgs): { update_config: UpdateConfigArgs } => { return { update_config: args }; }
  withdraw = async(sender:string, args: WithdrawArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, this.withdrawMsg(args), fee || "auto", memo, funds);
  }
  withdrawMsg = (args: WithdrawArgs): { withdraw: WithdrawArgs } => { return { withdraw: args }; }
  unbond = async(sender:string, args: UnbondArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, this.unbondMsg(args), fee || "auto", memo, funds);
  }
  unbondMsg = (args: UnbondArgs): { unbond: UnbondArgs } => { return { unbond: args }; }
  retry = async(sender:string, args: RetryArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, this.retryMsg(args), fee || "auto", memo, funds);
  }
  retryMsg = (args: RetryArgs): { retry: RetryArgs } => { return { retry: args }; }
  updateOwnership = async(sender:string, args: UpdateOwnershipArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, this.updateOwnershipMsg(args), fee || "auto", memo, funds);
  }
  updateOwnershipMsg = (args: UpdateOwnershipArgs): { update_ownership: UpdateOwnershipArgs } => { return { update_ownership: args }; }
}
