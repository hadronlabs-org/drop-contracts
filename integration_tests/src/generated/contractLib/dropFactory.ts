import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
import { Coin } from "@cosmjs/amino";
export interface InstantiateMsg {
  code_ids: CodeIds;
  remote_opts: RemoteOpts;
  salt: string;
  subdenom: string;
  token_metadata: DenomMetadata;
}
export interface CodeIds {
  core_code_id: number;
  distribution_code_id: number;
  puppeteer_code_id: number;
  rewards_manager_code_id: number;
  strategy_code_id: number;
  token_code_id: number;
  validators_set_code_id: number;
  withdrawal_manager_code_id: number;
  withdrawal_voucher_code_id: number;
}
export interface RemoteOpts {
  connection_id: string;
  denom: string;
  port_id: string;
  transfer_channel_id: string;
  update_period: number;
}
export interface DenomMetadata {
  /**
   * Even longer description, example: "The native staking token of the Cosmos Hub"
   */
  description: string;
  /**
   * Lowercase moniker to be displayed in clients, example: "atom"
   */
  display: string;
  /**
   * Number of decimals
   */
  exponent: number;
  /**
   * Descriptive token name, example: "Cosmos Hub Atom"
   */
  name: string;
  /**
   * Symbol to be displayed on exchanges, example: "ATOM"
   */
  symbol: string;
  /**
   * URI to a document that contains additional information
   */
  uri?: string | null;
  /**
   * SHA256 hash of a document pointed by URI
   */
  uri_hash?: string | null;
}
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
export type CallbackArgs = {
  post_init: {};
};
export type UpdateConfigArgs =
  | {
      core: ConfigOptional;
    }
  | {
      validators_set: ConfigOptional2;
    }
  | {
      puppeteer_fees: FeesMsg;
    };
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
export type ProxyArgs =
  | {
      validator_set: ValidatorSetMsg;
    }
  | {
      core: CoreMsg;
    };
export type ValidatorSetMsg =
  | {
      update_validators: {
        validators: ValidatorData[];
      };
    }
  | {
      update_validator: {
        validator: ValidatorData;
      };
    };
export type CoreMsg = {
  update_non_native_rewards_receivers: {
    items: NonNativeRewardsItem[];
  };
};
/**
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 *
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>. See also <https://github.com/CosmWasm/cosmwasm/blob/main/docs/MESSAGE_TYPES.md>.
 */
export type Binary = string;

export interface DropFactorySchema {
  responses: State;
  execute: InitArgs | CallbackArgs | UpdateConfigArgs | ProxyArgs | AdminExecuteArgs;
  [k: string]: unknown;
}
export interface State {
  core_contract: string;
  distribution_contract: string;
  puppeteer_contract: string;
  rewards_manager_contract: string;
  strategy_contract: string;
  token_contract: string;
  validators_set_contract: string;
  withdrawal_manager_contract: string;
  withdrawal_voucher_contract: string;
}
export interface InitArgs {
  base_denom: string;
  core_params: CoreParams;
}
export interface CoreParams {
  bond_limit?: Uint128 | null;
  channel: string;
  idle_min_interval: number;
  lsm_redeem_threshold: number;
  puppeteer_timeout: number;
  unbond_batch_switch_time: number;
  unbonding_period: number;
  unbonding_safe_period: number;
}
export interface ConfigOptional {
  base_denom?: string | null;
  bond_limit?: Uint128 | null;
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
export interface ConfigOptional2 {
  owner?: Addr | null;
  provider_proposals_contract?: Addr | null;
  stats_contract?: Addr | null;
}
export interface FeesMsg {
  ack_fee: Uint128;
  recv_fee: Uint128;
  register_fee: Uint128;
  timeout_fee: Uint128;
}
export interface ValidatorData {
  valoper_address: string;
  weight: number;
}
export interface NonNativeRewardsItem {
  address: string;
  denom: string;
  fee: Decimal;
  fee_address: string;
  min_amount: Uint128;
}
export interface AdminExecuteArgs {
  addr: string;
  msg: Binary;
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
  queryState = async(): Promise<State> => {
    return this.client.queryContractSmart(this.contractAddress, { state: {} });
  }
  init = async(sender:string, args: InitArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { init: args }, fee || "auto", memo, funds);
  }
  callback = async(sender:string, args: CallbackArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { callback: args }, fee || "auto", memo, funds);
  }
  updateConfig = async(sender:string, args: UpdateConfigArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { update_config: args }, fee || "auto", memo, funds);
  }
  proxy = async(sender:string, args: ProxyArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { proxy: args }, fee || "auto", memo, funds);
  }
  adminExecute = async(sender:string, args: AdminExecuteArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { admin_execute: args }, fee || "auto", memo, funds);
  }
}
