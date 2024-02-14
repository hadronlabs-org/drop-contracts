import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
export interface InstantiateMsg {
  allowed_senders: string[];
  connection_id: string;
  owner: string;
  port_id: string;
  remote_denom: string;
  update_period: number;
}
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
export type IcaState = "none" | "in_progress" | "registered" | "timeout";
export type ArrayOfTransfer = Transfer[];
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
export type ArrayOfUnbondingDelegation = UnbondingDelegation[];

export interface LidoPuppeteerAuthzSchema {
  responses: Config | DelegationsResponse | State | ArrayOfTransfer | ArrayOfUnbondingDelegation;
  execute:
    | RegisterDelegatorDelegationsQueryArgs
    | RegisterDelegatorUnbondingDelegationsQueryArgs
    | SetFeesArgs
    | DelegateArgs
    | UndelegateArgs
    | RedelegateArgs
    | TokenizeShareArgs
    | RedeemShareArgs;
  [k: string]: unknown;
}
export interface Config {
  allowed_senders: Addr[];
  connection_id: string;
  owner: Addr;
  port_id: string;
  proxy_address?: Addr | null;
  remote_denom: string;
  update_period: number;
}
export interface DelegationsResponse {
  delegations: Delegation[];
  last_updated_height: number;
}
/**
 * Delegation is basic (cheap to query) data about a delegation.
 *
 * Instances are created in the querier.
 */
export interface Delegation {
  /**
   * How much we have locked in the delegation
   */
  amount: Coin;
  delegator: Addr;
  /**
   * A validator address (e.g. cosmosvaloper1...)
   */
  validator: string;
  [k: string]: unknown;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export interface State {
  ica?: string | null;
  ica_state: IcaState;
  last_processed_height?: number | null;
}
export interface Transfer {
  amount: string;
  denom: string;
  recipient: string;
  sender: string;
}
export interface UnbondingDelegation {
  last_updated_height: number;
  query_id: number;
  unbonding_delegations: UnbondingEntry[];
  validator_address: string;
}
export interface UnbondingEntry {
  balance: Uint128;
  completion_time?: Timestamp | null;
  creation_height: number;
  initial_balance: Uint128;
  [k: string]: unknown;
}
export interface RegisterDelegatorDelegationsQueryArgs {
  validators: string[];
}
export interface RegisterDelegatorUnbondingDelegationsQueryArgs {
  validators: string[];
}
export interface SetFeesArgs {
  ack_fee: Uint128;
  recv_fee: Uint128;
  register_fee: Uint128;
  timeout_fee: Uint128;
}
export interface DelegateArgs {
  amount: Uint128;
  reply_to: string;
  timeout?: number | null;
  validator: string;
}
export interface UndelegateArgs {
  amount: Uint128;
  reply_to: string;
  timeout?: number | null;
  validator: string;
}
export interface RedelegateArgs {
  amount: Uint128;
  reply_to: string;
  timeout?: number | null;
  validator_from: string;
  validator_to: string;
}
export interface TokenizeShareArgs {
  amount: Uint128;
  reply_to: string;
  timeout?: number | null;
  validator: string;
}
export interface RedeemShareArgs {
  amount: Uint128;
  denom: string;
  reply_to: string;
  timeout?: number | null;
  validator: string;
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
  queryState = async(): Promise<State> => {
    return this.client.queryContractSmart(this.contractAddress, { state: {} });
  }
  queryTransactions = async(): Promise<ArrayOfTransfer> => {
    return this.client.queryContractSmart(this.contractAddress, { transactions: {} });
  }
  queryDelegations = async(): Promise<DelegationsResponse> => {
    return this.client.queryContractSmart(this.contractAddress, { delegations: {} });
  }
  queryUnbondingDelegations = async(): Promise<ArrayOfUnbondingDelegation> => {
    return this.client.queryContractSmart(this.contractAddress, { unbonding_delegations: {} });
  }
  registerICA = async(sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { register_i_c_a: {} }, fee || "auto", memo, funds);
  }
  registerQuery = async(sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { register_query: {} }, fee || "auto", memo, funds);
  }
  registerDelegatorDelegationsQuery = async(sender:string, args: RegisterDelegatorDelegationsQueryArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { register_delegator_delegations_query: args }, fee || "auto", memo, funds);
  }
  registerDelegatorUnbondingDelegationsQuery = async(sender:string, args: RegisterDelegatorUnbondingDelegationsQueryArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { register_delegator_unbonding_delegations_query: args }, fee || "auto", memo, funds);
  }
  setFees = async(sender:string, args: SetFeesArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { set_fees: args }, fee || "auto", memo, funds);
  }
  delegate = async(sender:string, args: DelegateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { delegate: args }, fee || "auto", memo, funds);
  }
  undelegate = async(sender:string, args: UndelegateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { undelegate: args }, fee || "auto", memo, funds);
  }
  redelegate = async(sender:string, args: RedelegateArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { redelegate: args }, fee || "auto", memo, funds);
  }
  tokenizeShare = async(sender:string, args: TokenizeShareArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { tokenize_share: args }, fee || "auto", memo, funds);
  }
  redeemShare = async(sender:string, args: RedeemShareArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { redeem_share: args }, fee || "auto", memo, funds);
  }
}
