import { CosmWasmClient, SigningCosmWasmClient, ExecuteResult, InstantiateResult } from "@cosmjs/cosmwasm-stargate"; 
import { StdFee } from "@cosmjs/amino";
export interface InstantiateMsg {
  base_denom: string;
  idle_min_interval: number;
  owner: string;
  pump_address?: string | null;
  puppeteer_contract: string;
  puppeteer_timeout: number;
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
export type Decimal1 = string;
export type UnbondBatchStatus = "new" | "unbonding" | "unbonded" | "withdrawn";

export interface LidoCoreSchema {
  responses: Config | Decimal | UnbondBatch;
  query: UnbondBatchArgs;
  execute: BondArgs | UpdateConfigArgs | FakeProcessBatchArgs;
  [k: string]: unknown;
}
export interface Config {
  base_denom: string;
  idle_min_interval: number;
  ld_denom?: string | null;
  owner: string;
  pump_address?: string | null;
  puppeteer_contract: string;
  puppeteer_timeout: number;
  strategy_contract: string;
  token_contract: string;
  unbond_batch_switch_time: number;
  unbonding_period: number;
  unbonding_safe_period: number;
  validators_set_contract: string;
  withdrawal_manager_contract: string;
  withdrawal_voucher_contract: string;
}
export interface UnbondBatch {
  created: number;
  expected_amount: Uint128;
  expected_release: number;
  slashing_effect?: Decimal1 | null;
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
  idle_min_interval?: number | null;
  ld_denom?: string | null;
  owner?: string | null;
  pump_address?: string | null;
  puppeteer_contract?: string | null;
  puppeteer_timeout?: number | null;
  strategy_contract?: string | null;
  token_contract?: string | null;
  unbond_batch_switch_time?: number | null;
  unbonding_period?: number | null;
  unbonding_safe_period?: number | null;
  validators_set_contract?: string | null;
  withdrawal_manager_contract?: string | null;
  withdrawal_voucher_contract?: string | null;
}
export interface FakeProcessBatchArgs {
  batch_id: Uint128;
  unbonded_amount: Uint128;
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
  queryExchangeRate = async(): Promise<Decimal> => {
    return this.client.queryContractSmart(this.contractAddress, { exchange_rate: {} });
  }
  queryUnbondBatch = async(args: UnbondBatchArgs): Promise<UnbondBatch> => {
    return this.client.queryContractSmart(this.contractAddress, { unbond_batch: args });
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
  fakeProcessBatch = async(sender:string, args: FakeProcessBatchArgs, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { fake_process_batch: args }, fee || "auto", memo, funds);
  }
  tick = async(sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { tick: {} }, fee || "auto", memo, funds);
  }
  puppeteerHook = async(sender: string, fee?: number | StdFee | "auto", memo?: string, funds?: Coin[]): Promise<ExecuteResult> =>  {
          if (!isSigningCosmWasmClient(this.client)) { throw this.mustBeSigningClient(); }
    return this.client.execute(sender, this.contractAddress, { puppeteer_hook: {} }, fee || "auto", memo, funds);
  }
}
